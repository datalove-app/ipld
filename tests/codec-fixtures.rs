use std::env;
use std::fs::{self, DirEntry};
use std::path::PathBuf;

use ipld::dev::MemoryContext;
use ipld::prelude::*;

static FIXTURE_SKIPLIST: &[(&str, &str)] = &[
    // skipped by libipld
    // ("int--11959030306112471732", "integer out of int64 range"),
    // (
    //     "dagpb_11unnamedlinks+data",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // ("dagpb_1link", "DAG-PB isn't fully compatible yet"),
    // ("dagpb_2link+data", "DAG-PB isn't fully compatible yet"),
    // (
    //     "dagpb_4namedlinks+data",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // (
    //     "dagpb_7unnamedlinks+data",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // ("dagpb_Data_zero", "DAG-PB isn't fully compatible yet"),
    // ("dagpb_empty", "DAG-PB isn't fully compatible yet"),
    // ("dagpb_Links_Hash_some", "DAG-PB isn't fully compatible yet"),
    // (
    //     "dagpb_Links_Hash_some_Name_some",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // (
    //     "dagpb_Links_Hash_some_Name_zero",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // (
    //     "dagpb_Links_Hash_some_Tsize_some",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // (
    //     "dagpb_Links_Hash_some_Tsize_zero",
    //     "DAG-PB isn't fully compatible yet",
    // ),
    // ("dagpb_simple_forms_2", "DAG-PB isn't fully compatible yet"),
    // ("dagpb_simple_forms_3", "DAG-PB isn't fully compatible yet"),
    // ("dagpb_simple_forms_4", "DAG-PB isn't fully compatible yet"),

    // skipped as we're not ready to test these yet
    ("array", "not ready"),
    ("bytes", "not ready"),
    ("cid", "not ready"),
    ("dagpb", "not yet implemented"),
    ("float", "not ready"),
    ("garbage", "lolwut"),
    ("int", "not ready"),
    ("map", "not ready"),
    ("string", "not ready"),
];

const DEFAULT_CID_VERSION: Version = Version::V1;
const DEFAULT_MH: u64 = Multihash::SHA2_256;

/// Contents of a single fixture.
#[derive(Debug)]
struct Fixture {
    codec: Multicodec,
    cid: Cid,
    block: Vec<u8>,
}

/// Returns all fixtures from a directory.
fn load_fixture(dir: DirEntry) -> Vec<Fixture> {
    fs::read_dir(&dir.path())
        .unwrap()
        .filter_map(|file| {
            // Filter out invalid files.
            let file = file.ok()?;
            let path = file.path();

            // codec
            let codec_str = path
                .extension()
                .expect("Filename must have an extension")
                .to_str()
                .expect("Extension must be valid UTF-8");
            // TODO: only using dag-json for now
            let codec = match codec_str {
                "dag-json" => Multicodec::DagJson(DagJson::new()),
                // "dag-cbor" => Multicodec::DagCbor(DagCbor::new()),
                // "dag-pb" => Codec::DagPb,
                _ => return None,
            };

            // cid
            let cid = path
                .file_stem()
                .expect("Filename must have a name")
                .to_str()
                .expect("Filename must be valid UTF-8")
                .try_into()
                .expect("Filename must be a valid Cid");

            // block bytes
            let block = fs::read(&path).expect("File must be able to be read");

            Some(Fixture { codec, cid, block })
        })
        .collect()
}

/// Returns the paths to all directories that contain fixtures.
fn fixture_directories() -> Vec<DirEntry> {
    let rust_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let mut fixtures_dir = PathBuf::from(rust_dir);
    fixtures_dir.push("codec-fixtures/fixtures");

    // Only take directories, exclude files
    fs::read_dir(&fixtures_dir)
        .expect("Cannot open fixtures directory")
        .filter_map(Result::ok)
        .filter(|dir| dir.path().is_dir())
        .collect()
}

/// Returns true if a test fixture is on the skip list
fn skip_test(dir: &DirEntry) -> bool {
    for (name, reason) in FIXTURE_SKIPLIST {
        if dir
            .path()
            .into_os_string()
            .to_str()
            .unwrap()
            .starts_with(name)
        {
            eprintln!("Skipping fixture '{}': {}", name, reason);
            return true;
        }
    }
    false
}

/// Sets up a `MemoryContext` to provide the fixture's block.
fn setup_ctx(fixture: &Fixture) -> MemoryContext {
    let mut ctx = MemoryContext::default();
    let cid = ctx
        .add_block(
            DEFAULT_CID_VERSION,
            fixture.codec.code(),
            DEFAULT_MH,
            fixture.block.clone(),
        )
        .expect("should not fail to add block");

    assert_eq!(&fixture.cid, &cid, "generated Cid should equal fixture Cid");
    ctx
}

#[test]
fn codec_fixtures() {
    for dir in fixture_directories() {
        if skip_test(&dir) {
            continue;
        }

        let fixture_name = dir
            .path()
            .file_stem()
            .expect("Directory must have a name")
            .to_os_string()
            .to_str()
            .expect("Filename must be valid UTF-8")
            .to_string();

        println!("Testing fixture {}", fixture_name);
        let mut test_count: usize = 0;
        let fixtures = load_fixture(dir);
        for fixture in fixtures.into_iter() {
            let mut ctx = setup_ctx(&fixture);

            // let mut dag: Any::Null;
            // let params = SelectionParams::new(fixture.cid);
            // Any::select(params, ctx).expect("should not fail to select");

            let dag: Null = SelectionParams::<'_, _, Null>::new(fixture.cid)
                .into_dag_iter(&mut ctx)
                .expect("should not fail selection")
                .next()
                .expect("should produce at least one dag")
                .dag
                .downcast()
                .expect("should not fail to downcast");

            /*
            // Take a fixture of one codec and…
            let decoded: Ipld = match &from_fixture.codec[..] {
                "dag-cbor" => {
                    serde_ipld_dagcbor::from_slice(&from_fixture.bytes).expect("Decoding must work")
                }
                _ => Codecs::get(&from_fixture.codec)
                    .decode(&from_fixture.bytes)
                    .expect("Decoding must work"),
            };

            // …transcode it into any other fixture.
            for to_fixture in &fixtures {
                let codec = Codecs::get(&to_fixture.codec);
                let data = match &to_fixture.codec[..] {
                    "dag-cbor" => serde_ipld_dagcbor::to_vec(&decoded).expect("Encoding must work"),
                    _ => codec.encode(&decoded).expect("Encoding must work"),
                };
                let digest = Code::Sha2_256.digest(&data);
                let cid = Cid::new_v1(codec.into(), digest);
                assert_eq!(
                    cid, to_fixture.cid,
                    "CIDs match for the data decoded from {} encoded as {}",
                    from_fixture.codec, to_fixture.codec
                );
            }
             */

            test_count += 1;
        }

        assert!(test_count > 0, "ran no tests!");
    }
}
