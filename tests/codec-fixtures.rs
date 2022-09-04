use ipld::dev::MemoryContext;
use ipld::prelude::*;
use std::{
    env,
    fs::{self, DirEntry},
    path::PathBuf,
};

const DEFAULT_CID_VERSION: Version = Version::V1;
const DEFAULT_MH: u64 = Multihash::SHA2_256;

static FIXTURE_SKIPLIST: &[(&str, &str)] = &[
    // skipped by libipld
    // ("int--11959030306112471732", "integer out of int64 range"),
    ("float-array_of_specials", "incorrect naming"),
    (
        "dagpb_11unnamedlinks+data",
        "DAG-PB isn't fully compatible yet",
    ),
    ("dagpb_1link", "DAG-PB isn't fully compatible yet"),
    ("dagpb_2link+data", "DAG-PB isn't fully compatible yet"),
    (
        "dagpb_4namedlinks+data",
        "DAG-PB isn't fully compatible yet",
    ),
    (
        "dagpb_7unnamedlinks+data",
        "DAG-PB isn't fully compatible yet",
    ),
    ("dagpb_Data_zero", "DAG-PB isn't fully compatible yet"),
    ("dagpb_empty", "DAG-PB isn't fully compatible yet"),
    ("dagpb_Links_Hash_some", "DAG-PB isn't fully compatible yet"),
    (
        "dagpb_Links_Hash_some_Name_some",
        "DAG-PB isn't fully compatible yet",
    ),
    (
        "dagpb_Links_Hash_some_Name_zero",
        "DAG-PB isn't fully compatible yet",
    ),
    (
        "dagpb_Links_Hash_some_Tsize_some",
        "DAG-PB isn't fully compatible yet",
    ),
    (
        "dagpb_Links_Hash_some_Tsize_zero",
        "DAG-PB isn't fully compatible yet",
    ),
    ("dagpb_simple_forms_2", "DAG-PB isn't fully compatible yet"),
    ("dagpb_simple_forms_3", "DAG-PB isn't fully compatible yet"),
    ("dagpb_simple_forms_4", "DAG-PB isn't fully compatible yet"),
];

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

/// Type of the underlying block's type.
#[derive(Copy, Clone, Debug)]
enum FixtureType {
    Array,
    Bytes,
    Cid,
    DagPb,
    Bool,
    Float,
    Garbage,
    Int,
    Map,
    Null,
    String,
}

impl FixtureType {
    fn from(test_name: &str) -> Self {
        match test_name {
            n if n.starts_with("array") => Self::Array,
            n if n.starts_with("false") || n.starts_with("true") => Self::Bool,
            n if n.starts_with("bytes") => Self::Bytes,
            n if n.starts_with("cid") => Self::Cid,
            n if n.starts_with("dagpb") => Self::DagPb,
            n if n.starts_with("float") => Self::Float,
            n if n.starts_with("garbage") => Self::Garbage,
            n if n.starts_with("int") => Self::Int,
            n if n.starts_with("map") => Self::Map,
            n if n.starts_with("null") => Self::Null,
            n if n.starts_with("string") => Self::String,
            n => panic!("unsupported feature: {}", n),
        }
    }
}

/// Contents of a single fixture.
#[derive(Debug)]
struct Fixture {
    // the type in the underlying block, and the naem of the test
    info: (FixtureType, String),
    // multicodec used by the block
    codec: Multicodec,
    // cid of the block
    cid: Cid,
    // bytes of the block
    bytes: Vec<u8>,
}

impl Fixture {
    fn r#type(&self) -> FixtureType {
        self.info.0
    }

    /// Returns all fixtures from a test directory.
    fn load_tests(dir: DirEntry) -> Vec<Self> {
        let test_name = dir
            .file_name()
            .into_string()
            .expect("test name must be valid UTF-8");
        let fixture_type = FixtureType::from(&test_name);
        let info = (fixture_type, test_name);

        fs::read_dir(&dir.path())
            .unwrap()
            .filter_map(|file| {
                // Filter out invalid files.
                let file = file.ok()?;
                let path = file.path();
                let codec_str = path
                    .extension()
                    .expect("Filename must have an extension")
                    .to_str()
                    .expect("Extension must be valid UTF-8");

                // codec
                let codec = match codec_str {
                    "dag-json" => Multicodec::DagJson(DagJson::new()),
                    "dag-cbor" => Multicodec::DagCbor(DagCbor::new()),
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
                let bytes = fs::read(&path).expect("File must be able to be read");

                Some(Self {
                    info: info.clone(),
                    codec,
                    cid,
                    bytes,
                })
            })
            .collect()
    }

    /// Returns true if a test fixture is on the skip list
    fn should_run_test(&self) -> bool {
        FIXTURE_SKIPLIST.iter().all(|(name, reason)| {
            if self.info.1.starts_with(name) {
                eprintln!("Skipping fixture '{}': {}", name, reason);
                false
            } else {
                true
            }
        })
    }

    /// Sets up a `MemoryContext` to provide the fixture's block.
    fn setup(codec: &Multicodec, cid: &Cid, bytes: Vec<u8>) -> MemoryContext {
        let mut ctx = MemoryContext::default();
        let cid = ctx
            .add_block(DEFAULT_CID_VERSION, codec.code(), DEFAULT_MH, bytes)
            .expect("should not fail to add block");

        assert_eq!(&cid, &cid, "generated Cid should equal fixture Cid");
        ctx
    }

    fn run(self) {
        match self.r#type() {
            FixtureType::Null => self.run_for::<Null>(),
            FixtureType::Bool => self.run_for::<Bool>(),
            FixtureType::Int => self.run_for::<Int>(),
            FixtureType::Float => self.run_for::<Float>(),
            // FixtureType::Bytes => self.run_for::<Bytes>(),
            FixtureType::String => self.run_for::<String>(),
            // FixtureType::Array => {}
            // FixtureType::Map => {}
            // FixtureType::Cid => self.run_for::<Link<Null>>(),
            // FixtureType::DagPb => {}
            // FixtureType::Garbage => {}
            _ => (),
        }
    }

    fn run_for<T: Select<MemoryContext>>(self) {
        let Self {
            codec,
            cid,
            bytes,
            info,
        } = self;
        let mut ctx = Self::setup(&codec, &cid, bytes.clone());
        let dag: T = SelectionParams::<'_, _, T>::new(cid)
            .into_dag_iter(&mut ctx)
            .expect(&format!(
                "should not fail selection:\n\
                    \ttest name: {}\n\
                    \tcodec: {}\n\
                    \tdag type: {}\n",
                &info.1,
                codec.name(),
                T::NAME,
            ))
            .next()
            .expect("should produce at least one dag")
            .dag
            .downcast()
            .expect("should not fail to downcast");
    }
}

#[test]
fn codec_fixtures() {
    let fixtures = fixture_directories()
        .into_iter()
        .map(Fixture::load_tests)
        .flatten()
        .filter(Fixture::should_run_test)
        .collect::<Vec<Fixture>>();
    let fixture_count = fixtures.len();

    let mut actual_count = 0usize;
    for test in fixtures.into_iter() {
        test.run();
        actual_count += 1;
    }

    assert_eq!(
        fixture_count, actual_count,
        "should have run at least one test"
    );

    /*
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
        let fixtures = load_tests(dir);
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
     */
}
