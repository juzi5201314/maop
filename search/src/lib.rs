use std::fs::create_dir_all;
use std::sync::Arc;

use anyhow::Context;
use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use tantivy::collector::DocSetCollector;
use tantivy::query::QueryParser;
use tantivy::schema::{
    IndexRecordOption, Schema, TextFieldIndexing, TextOptions,
    INDEXED, STORED,
};
use tantivy::Index;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

static INDEX: Lazy<Arc<RwLock<Index>>> = Lazy::new(|| Arc::new(RwLock::new(index().unwrap())));
static SCHEMA: Lazy<Schema> = Lazy::new(schema);


fn schema() -> Schema {
    let mut schema_builder = Schema::builder();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(CANG_JIE)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();

    schema_builder.add_u64_field("id", INDEXED | STORED);
    schema_builder.add_text_field("title", text_options.clone());
    schema_builder.add_text_field("content", text_options);

    schema_builder.build()
}

fn index() -> anyhow::Result<Index> {
    let dir = config::get_config_temp().data_path().join("tantivy");
    create_dir_all(&dir)?;
    Index::open_or_create(
        tantivy::directory::MmapDirectory::open(dir)?,
        SCHEMA.clone(),
    )
    .context("search::index")
}

pub fn reindex() {

}

#[test]
fn test_search() {
    macro_rules! doc(
        () => {
            {
                ($crate::Document::default())
            }
        }; // avoids a warning due to the useless `mut`.
        ($($field:expr => $value:expr),*) => {
            {
                let mut document = tantivy::Document::default();
                $(
                    document.add(tantivy::schema::FieldValue::new($field, $value.into()));
                )*
                document
            }
        };
        // if there is a trailing comma retry with the trailing comma stripped.
        ($($field:expr => $value:expr),+ ,) => {
            doc!( $( $field => $value ), *)
        };
    );

    let schema = SCHEMA.clone();
    let index = Index::create_in_ram(schema.clone());
    index.tokenizers().register(
        cang_jie::CANG_JIE,
        CangJieTokenizer {
            worker: Arc::new(Jieba::new()),
            option: TokenizerOption::Unicode,
        },
    );

    let title = schema.get_field("title").unwrap();
    let content = schema.get_field("content").unwrap();
    let id = schema.get_field("id").unwrap();

    let mut writer = index.writer(1 << 24).unwrap();
    writer.add_document(doc!(
        id => 10u64,
        title => "标题Title1",
        content => "内容Content1",
    ));
    writer.add_document(doc!(
        id => 21u64,
        title => "标题Title for 2",
        content => "内容Content2",
    ));
    writer.add_document(doc!(
        id => 22u64,
        title => "标题Title2 for 2",
        content => "内容Content2",
    ));
    writer.commit().unwrap();

    let reader = index.reader().unwrap();

    let query_parser =
        QueryParser::for_index(&index, vec![title, content]);
    let query = query_parser.parse_query("1").unwrap();

    let searcher = reader.searcher();
    let res = searcher.search(&query, &DocSetCollector).unwrap();
    for doc_addr in res.into_iter() {
        let doc = searcher.doc(doc_addr).unwrap();
        dbg!(doc);
    }
}
