use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rusqlite::{Connection, params};

const DATASET_SIZES: &[usize] = &[10_000, 100_000, 1_000_000];
const QUERIES: &[&str] = &["document999", "doc", "dcm999", "missing-value"];

fn dataset(size: usize) -> Vec<String> {
    (0..size)
        .map(|index| format!("document{index:07}.txt"))
        .collect()
}

fn sqlite_index(items: &[String]) -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch("CREATE VIRTUAL TABLE items USING fts5(title, prefix='2 3 4');")
        .unwrap();
    let transaction = connection.transaction().unwrap();
    {
        let mut insert = transaction
            .prepare("INSERT INTO items(title) VALUES(?1)")
            .unwrap();
        for item in items {
            insert.execute([item]).unwrap();
        }
    }
    transaction.commit().unwrap();
    connection
}

fn custom_search(items: &[String], query: &str) -> usize {
    let query = query.to_ascii_lowercase();
    items
        .iter()
        .filter(|item| is_subsequence(&query, &item.to_ascii_lowercase()))
        .take(20)
        .count()
}

fn is_subsequence(query: &str, candidate: &str) -> bool {
    let mut characters = candidate.chars();
    query
        .chars()
        .all(|expected| characters.by_ref().any(|actual| actual == expected))
}

fn benchmark_indexes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("search-index");
    for &size in DATASET_SIZES {
        let items = dataset(size);
        let connection = sqlite_index(&items);
        group.throughput(Throughput::Elements(size as u64));
        for &query in QUERIES {
            group.bench_with_input(
                BenchmarkId::new(format!("custom/{query}"), size),
                &query,
                |bencher, query| {
                    bencher.iter(|| custom_search(&items, query));
                },
            );
            if !query.contains("dcm") {
                let fts_query = format!("\"{query}\"*");
                group.bench_with_input(BenchmarkId::new(format!("sqlite-fts5/{query}"), size), &fts_query, |bencher, query| {
                    bencher.iter(|| {
                        connection.query_row("SELECT count(*) FROM (SELECT rowid FROM items WHERE items MATCH ?1 LIMIT 20)", params![query], |row| row.get::<_, i64>(0)).unwrap()
                    });
                });
            }
        }
    }
    group.finish();
}

criterion_group!(benches, benchmark_indexes);
criterion_main!(benches);
