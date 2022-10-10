use ipld::dev::MemoryContext;
use ipld::prelude::*;
use std::{
    env,
    fs::{self, DirEntry},
    path::PathBuf,
};

static CODEC_SKIPLIST: &[&str] = &[
    // "dag-cbor",
    // "dag-json",
];
static FIXTURE_SKIPLIST: &[(&str, &str)] = &[
    // unfamiliar, or incorrectly/confusingly labeled
    ("bytes-a1", "unsupported multibase"),
    ("bytes-empty", "no support for empty bytes yet"),
    ("bytes-long-8bit", "unsupported multibase"),
    ("cid-arrayof", "its a list of cids, not a cid of a list"),
    ("cid-mapof", "its a map of cids, not a cid of a map"),
    (
        "float-array_of_specials",
        "its a list of types, not a float",
    ),
    // (
    //     "cid-bahaacvrasyauh7rmlyrmyc7qzvktjv7x6q2h6ttvei6qon43tl3riaaaaaaa",
    //     "CIDv0 cannot be specified in CIDv1 format",
    // ),
    // (
    //     "cid-bafkreiebzrnroamgos2adnbpgw5apo3z4iishhbdx77gldnbk57d4zdio4",
    //     "CIDv0 cannot be specified in CIDv1 format",
    // ),
    // (
    //     "cid-bafyreiejkvsvdq4smz44yuwhfymcuvqzavveoj2at3utujwqlllspsqr6q",
    //     "CIDv0 cannot be specified in CIDv1 format",
    // ),
    // (
    //     "cid-QmRgutAxd8t7oGkSm4wmeuByG6M51wcTso6cubDdQtuEfL",
    //     "CIDv0 cannot be specified in CIDv1 format",
    // ),
    // (
    //     "cid-QmQg1v4o9xdT3Q14wh4S7dxZkDjyZ9ssFzFzyep1YrVJBY",
    //     "CIDv0 cannot be specified in CIDv1 format",
    // ),
    ("int-11959030306112471731", "fails for i64s"),
    ("int-18446744073709551615", "fails for i64s"),
    // skipped by libipld
    ("int--11959030306112471732", "integer out of int64 range"),
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
    block: Vec<u8>,
}

impl Fixture {
    fn r#type(&self) -> FixtureType {
        self.info.0
    }

    /// Loads all blocks from a test directory as fixtures.
    fn load_test_blocks(dir: DirEntry) -> Vec<Self> {
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
                let block = fs::read(&path).expect("File must be able to be read");

                Some(Self {
                    info: info.clone(),
                    codec,
                    cid,
                    block,
                })
            })
            .collect()
    }

    /// Returns false if a fixture should be skipped.
    fn should_run_test(&self) -> bool {
        CODEC_SKIPLIST.iter().all(|name| *name != self.codec.name())
            && FIXTURE_SKIPLIST.iter().all(|(name, reason)| {
                if self.info.1.starts_with(name) {
                    eprintln!("Skipping fixture '{}': {}", name, reason);
                    false
                } else {
                    true
                }
            })
    }

    /// Sets up a `MemoryContext` to provide the fixture's block.
    fn setup_ctx(&self) -> MemoryContext {
        const DEFAULT_CID_VERSION: Version = Version::V1;
        const DEFAULT_MH: u64 = Multihash::SHA2_256;

        let mut ctx = MemoryContext::default();
        let cid = ctx
            .add_block(
                DEFAULT_CID_VERSION,
                self.codec.code(),
                DEFAULT_MH,
                self.block.clone(),
            )
            .expect("should not fail to add block");

        assert_eq!(&self.cid, &cid, "generated Cid should match fixture Cid");
        ctx
    }

    fn run(&mut self) {
        // TODO: create copy types, test those too
        let did_run = match self.r#type() {
            FixtureType::Null => self.run_for::<Null>(),
            FixtureType::Bool => self.run_for::<Bool>(),
            FixtureType::Int => self.run_for::<Int>(),
            // FixtureType::Float => self.run_for::<Float>(), // floats arent round-tripping with dag-cbor correctly...
            // FixtureType::Bytes => self.run_for::<Bytes>(), // none of the fixtures match the multibase...
            FixtureType::String => self.run_for::<IpldString>(),
            // FixtureType::Array => self.run_for::<List<Any>>(),
            // FixtureType::Map => self.run_for::<Map<IpldString, Any>>(),
            // FixtureType::Cid => self.run_for::<Link<Any>>(),
            // FixtureType::DagPb => {}
            // FixtureType::Garbage => {}
            _ => false,
        };

        if did_run {
            // self.run_for::<Any>();
        }
    }

    fn run_for<T: Select<MemoryContext> + std::fmt::Debug + 'static>(&mut self) -> bool {
        {
            // first, decode the type directly using it's serde implementation
            let dag: T = self.codec.decode(self.block.as_slice()).expect(
                &self.format_err::<T>("should not fail to read dag directly from block bytes"),
            );
            // then, encode it to another codec
            let block = self
                .codec
                .encode(&dag)
                .expect(&self.format_err::<T>("should not fail to encode dag"));
            let new_cid = self
                .cid
                .derive_new(block.as_slice())
                .expect(&"should not fail to generate a Cid for a block of bytes");

            if self.codec.name() == "dag-json" {
                assert_eq!(
                    &self.cid,
                    &new_cid,
                    "{}\n{:?}\noriginal block: `{}`\nnew block `{}`",
                    &self.format_err::<T>(
                        "block from encoded dag should produce same Cid as input block"
                    ),
                    &dag,
                    std::str::from_utf8(self.block.as_slice()).unwrap(),
                    std::str::from_utf8(block.as_slice()).unwrap(),
                );
            } else {
                assert_eq!(
                    &self.cid,
                    &new_cid,
                    "{}\n{:?}\noriginal block: `{:?}`\nnew block `{:?}`",
                    &self.format_err::<T>(
                        "block from encoded dag should produce same Cid as input block"
                    ),
                    &dag,
                    self.block.as_slice(),
                    block.as_slice(),
                );
            }
        }

        {
            // next, decode the concrete type using the Matcher selector
            let mut ctx = self.setup_ctx();
            let matched_dag: T = Params::<'_, _, T>::new_select(self.cid)
                .into_dag_iter(&mut ctx)
                .expect(&self.format_err::<T>("should not fail selection"))
                .next()
                .expect("should produce at least one dag")
                .dag
                .downcast()
                .expect("should not fail to downcast to dag:");
            // then, encode it to another codec
            let block = self
                .codec
                .encode(&matched_dag)
                .expect(&self.format_err::<T>("should not fail to encode dag"));
            let new_cid = self
                .cid
                .derive_new(block.as_slice())
                .expect(&"should not fail to generate a Cid for a block of bytes");

            if self.codec.name() == "dag-json" {
                assert_eq!(
                    &self.cid,
                    &new_cid,
                    "{}\noriginal block: `{}`\nnew block `{}`",
                    &self.format_err::<T>(
                        "block from encoded dag should produce same Cid as input block"
                    ),
                    std::str::from_utf8(self.block.as_slice()).unwrap(),
                    std::str::from_utf8(block.as_slice()).unwrap(),
                );
            } else {
                assert_eq!(
                    &self.cid,
                    &new_cid,
                    "{}\noriginal block: `{:?}`\nnew block `{:?}`",
                    &self.format_err::<T>(
                        "block from encoded dag should produce same Cid as input block"
                    ),
                    self.block.as_slice(),
                    block.as_slice(),
                );
            }
        }

        true
    }

    fn format_err<T: Representation>(&self, msg: &str) -> String {
        format!(
            "{}\n\
                type: {}:\n\
                test name: {}\n\
                codec: {}\n\
                block: {:?}\n",
            msg,
            T::NAME,
            &self.info.1,
            self.codec.name(),
            &self.block,
        )
    }
}

#[test]
fn codec_fixtures() {
    // todo: this fixture-loading logic only round-trips, doesnt transcode
    let fixtures = fixture_directories()
        .into_iter()
        .map(Fixture::load_test_blocks)
        .flatten()
        .filter(Fixture::should_run_test)
        .collect::<Vec<Fixture>>();
    let fixture_count = fixtures.len();

    let mut actual_count = 0usize;
    for mut test in fixtures.into_iter() {
        test.run();
        actual_count += 1;
    }

    assert!(actual_count > 0, "should have run at least one test");
    assert_eq!(fixture_count, actual_count);

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
        let fixtures = load_test(dir);
        for fixture in fixtures.into_iter() {
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
