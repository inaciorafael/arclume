<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { loadPreferences, savePreferences } from "./services/preferences";
import { executeResult, searchApplications } from "./services/search";
import { GLOBAL_SHORTCUT_OPTIONS, getGlobalShortcut, setGlobalShortcut } from "./services/shortcut";
import { addIndexRoot, getIndexRoots, removeIndexRoot } from "./services/indexing";
import { getApplicationIcon } from "./services/icons";
import { clearClipboardHistory, getClipboardImage, getClipboardSettings, listClipboardItems, restoreClipboardItem, setClipboardSettings, type ClipboardItem, type ClipboardSettings } from "./services/clipboard";
import type { DictionaryEntry, SearchDiagnostics, SearchResult } from "./types/search";

const input = ref<HTMLInputElement>();
const settingsDialog = ref<HTMLDialogElement>();
const settingsClose = ref<HTMLButtonElement>();
const query = ref("");
const activeView = ref<"search" | "clipboard">("search");
const clipboardItems = ref<ClipboardItem[]>([]);
const clipboardSelectedIndex = ref(0);
const clipboardImage = ref("");
const clipboardSettings = reactive<ClipboardSettings>({ enabled: false, maxItems: 100, maxTotalBytes: 50 * 1024 * 1024, retentionDays: 7 });
const globalShortcut = ref("CmdOrControl+Space");
const indexRoots = ref<string[]>([]);
const newIndexRoot = ref("");
const results = ref<SearchResult[]>([]);
const applicationIcons = reactive(new Map<string, string>());
const pendingIcons = new Set<string>();
let loadingIcons = false;
const selectedIndex = ref(0);
const latestQueryId = ref(0);
let pendingSearch: { queryId: number; value: string } | undefined;
let searchInFlight = false;
let searchTimer: ReturnType<typeof setTimeout> | undefined;
const elapsedMicros = ref<number>();
const diagnostics = ref<SearchDiagnostics>();
const performanceSamples = ref<Array<{ roundTripMicros: number; renderMicros: number }>>([]);
const errorMessage = ref("");
const confirmationId = ref<string>();
const dictionaryEntry = ref<DictionaryEntry>();
const dictionaryLoading = ref(false);
const settingsOpen = ref(false);
const preferences = reactive(loadPreferences());
const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");

const selectedResult = computed(() => results.value[selectedIndex.value]);
const selectedClipboardItem = computed(() => clipboardItems.value[clipboardSelectedIndex.value]);
const selectedOptionId = computed(() => selectedResult.value ? `result-${selectedIndex.value}` : undefined);
const statusMessage = computed(() => {
  if (errorMessage.value) return errorMessage.value;
  if (confirmationId.value) return "Confirmation required. Press Enter again to continue, or Escape to cancel.";
  return results.value.length === 1 ? "1 result available" : `${results.value.length} results available`;
});
const performancePercentiles = computed(() => {
  const values = performanceSamples.value.map((sample) => sample.roundTripMicros).sort((left, right) => left - right);
  const percentile = (value: number) => values.length ? values[Math.max(0, Math.ceil(values.length * value) - 1)] : 0;
  return { count: values.length, p50: percentile(0.5), p95: percentile(0.95), p99: percentile(0.99) };
});
const latestRenderMicros = computed(() => performanceSamples.value[performanceSamples.value.length - 1]?.renderMicros ?? 0);

function applyPreferences() {
  const resolvedTheme = preferences.theme === "system" ? (systemTheme.matches ? "dark" : "light") : preferences.theme;
  document.documentElement.dataset.theme = resolvedTheme;
  document.documentElement.dataset.density = preferences.density;
  document.documentElement.classList.toggle("reduce-transparency", preferences.reduceTransparency);
}

watch(preferences, () => {
  savePreferences(preferences);
  applyPreferences();
}, { deep: true });

async function runSearch(queryId: number, value: string) {
  const requestStartedAt = performance.now();
  try {
    const response = await searchApplications(queryId, value);
    if (response.queryId !== latestQueryId.value) return;
    const responseReceivedAt = performance.now();
    results.value = response.results;
    queueApplicationIcons(response.results);
    selectedIndex.value = 0;
    confirmationId.value = undefined;
    elapsedMicros.value = response.elapsedMicros;
    diagnostics.value = response.diagnostics;
    errorMessage.value = "";
    await nextTick();
    performanceSamples.value.push({
      roundTripMicros: (responseReceivedAt - requestStartedAt) * 1000,
      renderMicros: (performance.now() - responseReceivedAt) * 1000,
    });
    if (performanceSamples.value.length > 200) performanceSamples.value.shift();
  } catch (error) {
    if (queryId !== latestQueryId.value) return;
    results.value = [];
    errorMessage.value = String(error);
  }
}

function queueApplicationIcons(items: SearchResult[]) {
  for (const item of items) {
    if (item.kind === "application" && !applicationIcons.has(item.id)) pendingIcons.add(item.id);
  }
  if (!loadingIcons) void loadPendingIcons();
}

async function loadPendingIcons() {
  loadingIcons = true;
  try {
    while (pendingIcons.size) {
      const id = pendingIcons.values().next().value as string;
      pendingIcons.delete(id);
      try {
        const icon = await getApplicationIcon(id);
        if (icon) applicationIcons.set(id, icon);
      } catch {
        // A missing OS icon keeps the deterministic category fallback.
      }
    }
  } finally {
    loadingIcons = false;
  }
}

async function drainSearchQueue() {
  searchTimer = undefined;
  if (searchInFlight || !pendingSearch) return;
  const request = pendingSearch;
  pendingSearch = undefined;
  searchInFlight = true;
  try {
    await runSearch(request.queryId, request.value);
  } finally {
    searchInFlight = false;
    if (pendingSearch) void drainSearchQueue();
  }
}

function scheduleSearch(value: string, immediate = false) {
  pendingSearch = { queryId: ++latestQueryId.value, value };
  if (searchTimer !== undefined) clearTimeout(searchTimer);
  if (immediate) {
    void drainSearchQueue();
    return;
  }
  searchTimer = setTimeout(() => void drainSearchQueue(), 16);
}

watch(query, (value) => {
  dictionaryEntry.value = undefined;
  scheduleSearch(value);
});
watch(selectedClipboardItem, async (item) => {
  clipboardImage.value = "";
  if (item?.kind === "image") {
    try { clipboardImage.value = await getClipboardImage(item.id); } catch { /* Item may have expired during selection. */ }
  }
});

function moveSelection(offset: number) {
  if (!results.value.length) return;
  selectedIndex.value = (selectedIndex.value + offset + results.value.length) % results.value.length;
  confirmationId.value = undefined;
  void scrollSelectedIntoView();
}

function selectBoundary(position: "first" | "last") {
  if (!results.value.length) return;
  selectedIndex.value = position === "first" ? 0 : results.value.length - 1;
  confirmationId.value = undefined;
  void scrollSelectedIntoView();
}

async function scrollSelectedIntoView() {
  await nextTick();
  document.getElementById(`result-${selectedIndex.value}`)?.scrollIntoView({ block: "nearest" });
}

async function hideLauncher() {
  await getCurrentWindow().hide();
}

async function openClipboard() {
  activeView.value = "clipboard";
  clipboardSelectedIndex.value = 0;
  try {
    clipboardItems.value = await listClipboardItems(clipboardSettings.maxItems);
    errorMessage.value = "";
  } catch (error) { errorMessage.value = String(error); }
}

function closeClipboard() {
  activeView.value = "search";
  void nextTick(() => input.value?.focus());
}

async function restoreClipboardSelection() {
  const item = selectedClipboardItem.value;
  if (!item) return;
  try { await restoreClipboardItem(item.id); await hideLauncher(); }
  catch (error) { errorMessage.value = String(error); }
}

async function saveClipboardSettings() {
  try {
    Object.assign(clipboardSettings, await setClipboardSettings({ ...clipboardSettings }));
    if (activeView.value === "clipboard") await openClipboard();
    errorMessage.value = "";
  } catch (error) { errorMessage.value = String(error); }
}

async function enableClipboardHistory() {
  clipboardSettings.enabled = true;
  await saveClipboardSettings();
}

async function clearClipboard() {
  if (!window.confirm("Clear the local Arclume clipboard history? This cannot be undone.")) return;
  try { await clearClipboardHistory(); clipboardItems.value = []; clipboardImage.value = ""; errorMessage.value = ""; }
  catch (error) { errorMessage.value = String(error); }
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

async function executeSelection() {
  const selected = selectedResult.value;
  if (!selected) return;
  if (selected.requiresConfirmation && confirmationId.value !== selected.id) {
    confirmationId.value = selected.id;
    return;
  }
  try {
    dictionaryLoading.value = selected.id.startsWith("dictionary:");
    const entry = await executeResult(selected.id, query.value, selected.title, selected.kind, confirmationId.value === selected.id);
    if (entry) {
      dictionaryEntry.value = entry;
      errorMessage.value = "";
      return;
    }
    await hideLauncher();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    dictionaryLoading.value = false;
  }
}

async function openSettings() {
  settingsOpen.value = true;
  await nextTick();
  settingsDialog.value?.showModal();
  settingsClose.value?.focus();
}

async function changeGlobalShortcut() {
  const requested = globalShortcut.value;
  try {
    await setGlobalShortcut(requested);
    errorMessage.value = "";
  } catch (error) {
    globalShortcut.value = await getGlobalShortcut();
    errorMessage.value = String(error);
  }
}

async function addRoot() {
  const root = newIndexRoot.value.trim();
  if (!root) return;
  try {
    indexRoots.value = await addIndexRoot(root);
    newIndexRoot.value = "";
    errorMessage.value = "";
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function removeRoot(root: string) {
  try {
    indexRoots.value = await removeIndexRoot(root);
    errorMessage.value = "";
  } catch (error) {
    errorMessage.value = String(error);
  }
}

function closeSettings() {
  settingsDialog.value?.close();
}

function handleDialogClose() {
  settingsOpen.value = false;
  void nextTick(() => input.value?.focus());
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "v") {
    event.preventDefault();
    if (!settingsOpen.value) void openClipboard();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key === ",") {
    event.preventDefault();
    if (!settingsOpen.value) void openSettings();
    return;
  }
  if (settingsOpen.value) return;
  if (activeView.value === "clipboard") {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const offset = event.key === "ArrowDown" ? 1 : -1;
      if (clipboardItems.value.length) clipboardSelectedIndex.value = (clipboardSelectedIndex.value + offset + clipboardItems.value.length) % clipboardItems.value.length;
    } else if (event.key === "Enter") { event.preventDefault(); void restoreClipboardSelection(); }
    else if (event.key === "Escape") { event.preventDefault(); closeClipboard(); }
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    moveSelection(1);
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    moveSelection(-1);
  } else if (event.key === "Home") {
    event.preventDefault();
    selectBoundary("first");
  } else if (event.key === "End") {
    event.preventDefault();
    selectBoundary("last");
  } else if (event.key === "Enter") {
    event.preventDefault();
    void executeSelection();
  } else if (event.key === "Escape") {
    event.preventDefault();
    if (confirmationId.value) confirmationId.value = undefined;
    else if (dictionaryEntry.value) dictionaryEntry.value = undefined;
    else if (query.value) query.value = "";
    else void hideLauncher();
  }
}

function resultKindLabel(kind: SearchResult["kind"]) {
  return ({ application: "Application", file: "File", folder: "Folder", action: "Action", plugin: "Plugin" })[kind];
}

function onSystemThemeChange() {
  if (preferences.theme === "system") applyPreferences();
}

onMounted(async () => {
  applyPreferences();
  systemTheme.addEventListener("change", onSystemThemeChange);
  await nextTick();
  input.value?.focus();
  globalShortcut.value = await getGlobalShortcut();
  indexRoots.value = await getIndexRoots();
  Object.assign(clipboardSettings, await getClipboardSettings());
  scheduleSearch("", true);
});

onBeforeUnmount(() => {
  systemTheme.removeEventListener("change", onSystemThemeChange);
  if (searchTimer !== undefined) clearTimeout(searchTimer);
});
</script>

<template>
  <main class="launcher-shell" @keydown="handleKeydown">
    <section class="launcher" aria-label="Arclume launcher">
      <header class="search-bar">
        <span class="brand-mark" aria-hidden="true">A</span>
        <nav class="mode-tabs" aria-label="Arclume sections">
          <button type="button" :class="{ active: activeView === 'search' }" @click="closeClipboard">Search</button>
          <button type="button" :class="{ active: activeView === 'clipboard' }" @click="openClipboard">Clipboard <span v-if="clipboardItems.length" class="tab-count">{{ clipboardItems.length }}</span></button>
        </nav>
        <input v-if="activeView === 'search'"
          ref="input"
          v-model="query"
          role="combobox"
          aria-label="Search applications, files and actions"
          aria-autocomplete="list"
          aria-controls="search-results"
          :aria-activedescendant="selectedOptionId"
          :aria-expanded="results.length > 0"
          autocomplete="off"
          placeholder="Search apps, files, actions and plugins"
          spellcheck="false"
        />
        <div v-else class="view-title"><span class="eyebrow">Local history</span><strong>Your latest copied items</strong></div>
        <button class="icon-button" aria-label="Open settings" title="Settings (Ctrl+,)" @click="openSettings">&#9881;</button>
        <kbd>Esc</kbd>
      </header>

      <template v-if="activeView === 'clipboard'">
        <div class="result-label"><span>{{ clipboardItems.length }} saved items</span><span>{{ clipboardSettings.enabled ? "Capture active" : "Capture paused" }}</span></div>
        <div class="content clipboard-content">
          <ul class="results clipboard-results" role="listbox" aria-label="Clipboard history">
            <li v-for="(item, index) in clipboardItems" :key="item.id" :class="{ selected: index === clipboardSelectedIndex }" role="option" :aria-selected="index === clipboardSelectedIndex" @mousemove="clipboardSelectedIndex = index" @dblclick="restoreClipboardSelection">
              <span class="result-icon clipboard-kind" :data-kind="item.kind" aria-hidden="true">{{ item.kind === "image" ? "▧" : "T" }}</span>
              <span class="result-copy"><strong>{{ item.preview }}</strong><small>{{ formatBytes(item.byteSize) }} · {{ new Date(item.capturedAt * 1000).toLocaleString() }}</small></span>
              <kbd v-if="index === clipboardSelectedIndex">Enter</kbd>
            </li>
            <li v-if="!clipboardItems.length" class="empty-state">
              <span>{{ clipboardSettings.enabled ? "Copy text or an image and it will appear here." : "Clipboard history is private and disabled by default." }}</span>
              <button v-if="!clipboardSettings.enabled" type="button" @click="enableClipboardHistory">Enable history</button>
            </li>
          </ul>
          <aside class="preview clipboard-preview" aria-label="Clipboard item preview">
            <template v-if="selectedClipboardItem">
              <span class="kind-pill">{{ selectedClipboardItem.kind }}</span>
              <img v-if="clipboardImage" :src="clipboardImage" alt="Copied image preview" />
              <p v-else class="clipboard-text">{{ selectedClipboardItem.preview }}</p>
              <div class="preview-action"><span>Copy again</span><kbd>Enter</kbd></div>
            </template>
          </aside>
        </div>
      </template>
      <template v-else>
      <div class="result-label">
        <span>{{ results.length ? (query ? "Results" : "Recent") : errorMessage || "No results found" }}</span>
        <span v-if="preferences.showLatency && elapsedMicros !== undefined" class="latency">{{ (elapsedMicros / 1000).toFixed(2) }} ms</span>
      </div>
      </template>

      <div class="content" :class="{ 'preview-hidden': !preferences.showPreview }">
        <ul id="search-results" class="results" role="listbox" aria-label="Search results">
          <li
            v-for="(result, index) in results"
            :id="`result-${index}`"
            :key="result.id"
            :aria-selected="index === selectedIndex"
            :class="{ selected: index === selectedIndex }"
            role="option"
            @mousemove="selectedIndex = index"
            @click="selectedIndex = index"
            @dblclick="executeSelection"
          >
            <span class="result-icon" :data-kind="result.kind" aria-hidden="true"><img v-if="applicationIcons.get(result.id)" :src="applicationIcons.get(result.id)" alt="" /></span>
            <span class="result-copy">
              <strong>{{ result.title }}</strong>
              <small v-if="confirmationId === result.id" class="confirmation">Press Enter again to confirm · Esc cancels</small>
              <small v-else>{{ result.subtitle }}</small>
            </span>
            <kbd v-if="index === selectedIndex">Enter</kbd>
          </li>
        </ul>

        <aside v-if="preferences.showPreview" class="preview" aria-label="Selected result preview">
          <template v-if="dictionaryEntry && selectedResult?.id.startsWith('dictionary:')">
            <span class="kind-pill">Dicionário</span>
            <h2>{{ dictionaryEntry.word }}</h2>
            <ol class="dictionary-definitions">
              <li v-for="definition in dictionaryEntry.definitions" :key="definition">{{ definition }}</li>
            </ol>
            <p class="dictionary-source">{{ dictionaryEntry.source }}<span v-if="dictionaryEntry.cached"> · cache local</span></p>
            <div class="preview-action"><span>Esc para voltar</span><kbd>Esc</kbd></div>
          </template>
          <template v-else-if="selectedResult">
            <span class="preview-icon result-icon" :data-kind="selectedResult.kind" aria-hidden="true"><img v-if="applicationIcons.get(selectedResult.id)" :src="applicationIcons.get(selectedResult.id)" alt="" /></span>
            <span class="kind-pill">{{ resultKindLabel(selectedResult.kind) }}</span>
            <h2>{{ selectedResult.title }}</h2>
            <p>{{ selectedResult.subtitle }}</p>
            <div class="preview-action">
              <span>{{ dictionaryLoading ? "Consultando…" : selectedResult.requiresConfirmation ? "Confirmation required" : "Ready to open" }}</span>
              <kbd>Enter</kbd>
            </div>
            <section v-if="preferences.showPerformancePanel && diagnostics" class="performance-panel" aria-label="Local performance diagnostics">
              <div class="performance-heading"><strong>Performance</strong><span>{{ performancePercentiles.count }}/200 samples</span></div>
              <dl>
                <div><dt>Backend</dt><dd>{{ (elapsedMicros! / 1000).toFixed(2) }} ms</dd></div>
                <div><dt>Files</dt><dd>{{ (diagnostics.fileProviderMicros / 1000).toFixed(2) }} ms</dd></div>
                <div><dt>Plugins</dt><dd>{{ (diagnostics.pluginProviderMicros / 1000).toFixed(2) }} ms</dd></div>
                <div><dt>Ranking</dt><dd>{{ (diagnostics.rankingMicros / 1000).toFixed(2) }} ms</dd></div>
                <div><dt>Render</dt><dd>{{ (latestRenderMicros / 1000).toFixed(2) }} ms</dd></div>
                <div><dt>p50 / p95 / p99</dt><dd>{{ (performancePercentiles.p50 / 1000).toFixed(1) }} / {{ (performancePercentiles.p95 / 1000).toFixed(1) }} / {{ (performancePercentiles.p99 / 1000).toFixed(1) }} ms</dd></div>
              </dl>
            </section>
          </template>
          <p v-else class="preview-empty">Start typing to find something.</p>
        </aside>
      </div>

      <footer>
        <span><kbd>&uarr;</kbd><kbd>&darr;</kbd> Navigate</span>
        <span><kbd>&crarr;</kbd> Open</span>
        <span><kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>V</kbd> Clipboard</span>
        <button class="footer-button" @click="openSettings"><kbd>Ctrl</kbd><kbd>,</kbd> Settings</button>
      </footer>
    </section>

    <p class="sr-only" role="status" aria-live="polite">{{ statusMessage }}</p>

    <dialog
      v-if="settingsOpen"
      ref="settingsDialog"
      class="settings-dialog"
      aria-labelledby="settings-title"
      @cancel.prevent="closeSettings"
      @close="handleDialogClose"
      @keydown.esc.prevent.stop="closeSettings"
    >
      <header>
        <div>
          <span class="eyebrow">Preferences</span>
          <h2 id="settings-title">Make Arclume yours</h2>
        </div>
        <button ref="settingsClose" class="icon-button" aria-label="Close settings" @click="closeSettings">&times;</button>
      </header>
      <div class="settings-grid">
        <label>
          <span>Theme</span>
          <select v-model="preferences.theme">
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
        <label>
          <span>Result density</span>
          <select v-model="preferences.density">
            <option value="comfortable">Comfortable</option>
            <option value="compact">Compact</option>
          </select>
        </label>
        <label>
          <span>Global shortcut</span>
          <select v-model="globalShortcut" @change="changeGlobalShortcut">
            <option v-for="option in GLOBAL_SHORTCUT_OPTIONS" :key="option.value" :value="option.value">{{ option.label }}</option>
          </select>
        </label>
        <label class="toggle"><input v-model="preferences.showPreview" type="checkbox" /> <span>Show contextual preview</span></label>
        <label class="toggle"><input v-model="preferences.showLatency" type="checkbox" /> <span>Show search latency</span></label>
        <label class="toggle"><input v-model="preferences.showPerformancePanel" type="checkbox" /> <span>Show local performance panel</span></label>
        <label class="toggle"><input v-model="preferences.reduceTransparency" type="checkbox" /> <span>Reduce transparency</span></label>
        <section class="clipboard-settings" aria-labelledby="clipboard-settings-title">
          <div><strong id="clipboard-settings-title">Clipboard history</strong><small>Opt-in and stored only on this device. Arclume captures while it is running.</small></div>
          <label class="toggle"><input v-model="clipboardSettings.enabled" type="checkbox" @change="saveClipboardSettings" /> <span>Capture copied text and images</span></label>
          <label><span>Maximum items</span><select v-model.number="clipboardSettings.maxItems" @change="saveClipboardSettings"><option :value="25">25</option><option :value="50">50</option><option :value="100">100</option><option :value="250">250</option><option :value="500">500</option></select></label>
          <label><span>Disk limit</span><select v-model.number="clipboardSettings.maxTotalBytes" @change="saveClipboardSettings"><option :value="10 * 1024 * 1024">10 MB</option><option :value="50 * 1024 * 1024">50 MB</option><option :value="100 * 1024 * 1024">100 MB</option><option :value="250 * 1024 * 1024">250 MB</option></select></label>
          <label><span>Retention</span><select v-model.number="clipboardSettings.retentionDays" @change="saveClipboardSettings"><option :value="1">1 day</option><option :value="7">7 days</option><option :value="30">30 days</option><option :value="90">90 days</option></select></label>
          <button type="button" class="danger-button" @click="clearClipboard">Clear clipboard history</button>
        </section>
        <section class="index-roots" aria-labelledby="index-roots-title">
          <div><strong id="index-roots-title">Indexed folders</strong><small>Add an absolute folder such as C:\Projects. C:\ is supported but system folders are excluded.</small></div>
          <form @submit.prevent="addRoot">
            <input v-model="newIndexRoot" aria-label="Folder path to index" placeholder="C:\Projects" spellcheck="false" />
            <button type="submit">Add</button>
          </form>
          <ul>
            <li v-for="root in indexRoots" :key="root"><span :title="root">{{ root }}</span><button type="button" :aria-label="`Stop indexing ${root}`" @click="removeRoot(root)">&times;</button></li>
          </ul>
        </section>
      </div>
      <p class="settings-note">Preferences and index roots stay on this device. Newly added folders are indexed in the background.</p>
    </dialog>
  </main>
</template>
