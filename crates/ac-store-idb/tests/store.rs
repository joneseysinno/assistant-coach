//! IdbStore: the shared Store contract, fixture round-trip through InfiniteDB (S14-T1),
//! persistence across reopen, and save latency.

use std::path::Path;
use std::time::Instant;

use ac_domain::store::{contract, Doc, DocKind, Store};
use ac_domain::{Game, Play, Practice};
use ac_store_idb::IdbStore;

#[test]
fn idb_store_meets_the_contract() {
    let dir = tempfile::TempDir::new().unwrap();
    contract(&IdbStore::open(dir.path()).unwrap());
}

fn fixtures() -> Vec<Doc> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures");
    let mut docs = Vec::new();
    for sub in ["games", "games/stress", "plays", "practices"] {
        for e in std::fs::read_dir(root.join(sub)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_none_or(|x| x != "yaml") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            docs.push(match sub {
                "plays" => Doc::Play(ac_domain::from_yaml::<Play>(&text).unwrap()),
                "practices" => Doc::Practice(ac_domain::from_yaml::<Practice>(&text).unwrap()),
                _ => Doc::Game(ac_domain::from_yaml::<Game>(&text).unwrap()),
            });
        }
    }
    docs
}

/// S14-T1 (+ D-018): every fixture → save → reopen the database → load → equal.
#[test]
fn every_fixture_round_trips_through_infinitedb() {
    let dir = tempfile::TempDir::new().unwrap();
    let docs = fixtures();
    assert_eq!(docs.len(), 16);
    {
        let store = IdbStore::open(dir.path()).unwrap();
        for d in &docs {
            store.save(d).unwrap();
        }
    }
    let store = IdbStore::open(dir.path()).unwrap();
    assert_eq!(store.len(), 16);
    for d in &docs {
        assert_eq!(
            store.load(d.kind(), d.id()).unwrap().as_ref(),
            Some(d),
            "{}",
            d.id()
        );
    }
    assert_eq!(store.list(Some(DocKind::Play)).unwrap().len(), 2);
}

#[test]
fn edits_and_deletes_survive_reopen() {
    let dir = tempfile::TempDir::new().unwrap();
    let mut g = Game {
        id: "g-1".into(),
        name: "First".into(),
        ..Default::default()
    };
    {
        let s = IdbStore::open(dir.path()).unwrap();
        s.save(&Doc::Game(g.clone())).unwrap();
        g.name = "Second".into();
        s.save(&Doc::Game(g.clone())).unwrap();
        s.save(&Doc::Game(Game {
            id: "g-2".into(),
            ..Default::default()
        }))
        .unwrap();
        s.delete(DocKind::Game, "g-2").unwrap();
    }
    let s = IdbStore::open(dir.path()).unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s.load(DocKind::Game, "g-1").unwrap(), Some(Doc::Game(g)));
    // a new save after reopen gets a fresh serial (doesn't overwrite g-1)
    s.save(&Doc::Game(Game {
        id: "g-3".into(),
        ..Default::default()
    }))
    .unwrap();
    drop(s);
    assert_eq!(IdbStore::open(dir.path()).unwrap().len(), 2);
}

/// How long one autosave takes (insert + sync). Printed, not asserted: the Studio
/// debounces saves, so this only needs to be well under a second.
#[test]
fn save_latency() {
    let dir = tempfile::TempDir::new().unwrap();
    let s = IdbStore::open(dir.path()).unwrap();
    let doc = fixtures().into_iter().next().unwrap();
    let mut times = Vec::new();
    for _ in 0..30 {
        let t = Instant::now();
        s.save(&doc).unwrap();
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "save latency ms: p50 {:.1}  p95 {:.1}  max {:.1}",
        times[15], times[28], times[29]
    );
    assert!(times[28] < 1000.0);
}
