use std::borrow::Cow;
use std::collections::HashMap;

use oxilangtag::LanguageTag;

use serde::Deserialize;
use serde::Serialize;
use warc::BufferedBody;
use warc::Record;
use warc::WarcHeader;

use crate::common::Identification as IdentificationGen;

type Identification = IdentificationGen<String>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]

/// OSCAR-specific metadata
// TODO: make it a HashMap
// TODO: make annotation/categories hashmaps
/// Contains document metadata:
/// - `identification` is the document-level language identification (see [Identification])
/// - `harmful_pp` is the perplexiry of the document, related to a model trained to recognize adult documents
/// - `quality_warnings` (ex-annotation) contains tags for some length/content based quality filters
/// - `categories` contains categories based on the url of the document. Uses the ut1 blocklist as a base.
/// - `sentence_identifiations` contains line-level identifications.
pub struct Metadata {
    identification: Identification,
    harmful_pp: Option<f32>,
    quality_warnings: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    sentence_identifications: Vec<Option<Identification>>,
}

impl Metadata {
    pub fn new(
        identification: &Identification,
        sentence_identifications: &[Option<Identification>],
    ) -> Self {
        Metadata {
            identification: identification.clone(),
            harmful_pp: None,
            quality_warnings: None,
            categories: None,
            sentence_identifications: sentence_identifications.to_owned(),
        }
    }

    pub fn add_annotation(&mut self, annotation: String) {
        match &mut self.quality_warnings {
            Some(anno) => anno.push(annotation),
            None => self.quality_warnings = Some(vec![annotation]),
        }
    }

    pub fn categories(&self) -> Option<&Vec<String>> {
        self.categories.as_ref()
    }
    pub fn add_category(&mut self, category: String) {
        match &mut self.categories {
            Some(cat) => cat.push(category),
            None => self.categories = Some(vec![category]),
        }
    }
    pub fn set_categories(&mut self, categories: Option<Vec<String>>) {
        self.categories = categories;
    }

    /// Get a reference to the metadata's annotation.
    pub fn annotation(&self) -> Option<&Vec<String>> {
        self.quality_warnings.as_ref()
    }

    /// Get a reference to the metadata's sentence identifications.
    pub fn sentence_identifications(&self) -> &[Option<Identification>] {
        self.sentence_identifications.as_ref()
    }

    pub fn set_harmful_pp(&mut self, harmful_pp: Option<f32>) {
        self.harmful_pp = harmful_pp;
    }
}

impl Default for Metadata {
    /// default Metadata is English with 1.0 prob,
    /// no annotation and a single english sentence with 1.0 prob.
    // TODO: Change to empty document with no id and no sentence id?
    fn default() -> Self {
        Self {
            identification: Identification::new(LanguageTag::parse("en".to_string()).unwrap(), 1.0),
            harmful_pp: None,
            quality_warnings: None,
            categories: None,
            sentence_identifications: vec![Some(Identification::new(
                LanguageTag::parse("en".to_string()).unwrap(),
                1.0,
            ))],
        }
    }
}

// TODO make this a newtype so that getters can do the u8 conversion
pub type WarcHeaders = HashMap<WarcHeader, Vec<u8>>;

pub type WarcHeadersSer = HashMap<WarcHeader, String>;

/// A Document is a structure holding content, WARC headers and OSCAR-specific metadata.
/// - TODO: Change warc_headers from [RawRecordHeader] to [warc::Record] with [warc::EmptyBody]?
/// This way we shouldn't have to parse strings or use unwrap on [RawRecordHeader].
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(from = "DocumentSer", into = "DocumentSer")]
pub struct Document {
    content: String,
    warc_headers: WarcHeaders,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize)]
/// Serializable version of [Document].
struct DocumentSer {
    content: String,
    warc_headers: WarcHeadersSer,
    metadata: Metadata,
}

impl DocumentSer {
    // pub fn get_schema() -> Result<String, Error> {
    //     serde_json::to_string_pretty(&schemars::schema_for!(Self)).map_err(Error::Serde)
    // }
}
impl From<Document> for DocumentSer {
    fn from(d: Document) -> Self {
        let warc_headers = d
            .warc_headers
            .into_iter()
            .map(|(k, v)| (k, String::from_utf8_lossy(&v).into_owned()))
            .collect();

        Self {
            content: d.content,
            warc_headers,
            metadata: d.metadata,
        }
    }
}

impl From<DocumentSer> for Document {
    fn from(d: DocumentSer) -> Self {
        let warc_headers = d
            .warc_headers
            .into_iter()
            .map(|(k, v)| (k, v.as_bytes().to_vec()))
            .collect();

        Self {
            content: d.content,
            warc_headers,
            metadata: d.metadata,
        }
    }
}

impl Document {
    pub fn new(content: String, warc_headers: WarcHeaders, metadata: Metadata) -> Self {
        Self {
            content,
            warc_headers,
            metadata,
        }
    }

    // pub fn get_schema() -> Result<String, Error> {
    //     DocumentSer::get_schema()
    // }
    /// Instantiate a Document from a record and a related metadata.
    pub fn from_record(record: Record<BufferedBody>, metadata: Metadata) -> Self {
        let (header, body) = record.into_raw_parts();
        let content = String::from_utf8_lossy(&body).into_owned();
        let warc_headers = header.headers;

        Self {
            content,
            warc_headers,
            metadata,
        }
    }

    /// Get a reference to the Document's identification
    pub fn identification(&self) -> &Identification {
        &self.metadata.identification
    }

    /// Get a reference to the content
    pub fn content(&self) -> &String {
        &self.content
    }

    /// get warc record id
    pub fn warc_id(&self) -> Cow<str> {
        String::from_utf8_lossy(self.warc_headers.get(&WarcHeader::RecordID).unwrap())
    }

    /// Get a reference to the document's warc headers.
    pub fn warc_headers(&self) -> &WarcHeaders {
        &self.warc_headers
    }

    /// shorthand to get url as a String
    pub fn url(&self) -> Option<String> {
        self.warc_headers()
            .get(&warc::WarcHeader::TargetURI)
            .map(|x| String::from_utf8_lossy(x).into_owned())
    }

    /// Get a mutable reference to the document's metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Get a reference to the document's metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Set the document's content.
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
}

/// custom debug implementation that converts:
/// - `headers` from [Vec<u8>] to [String] for easier readablility
/// - `content` from [String] to [Vec<String>] to better diagnose identification
impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers_pp: HashMap<WarcHeader, String> = self
            .warc_headers
            .iter()
            .map(|(k, v)| (k.clone(), String::from_utf8_lossy(v).to_string()))
            .collect();

        let lines = &self.content.lines().collect::<Vec<&str>>();
        f.debug_struct("Document")
            .field("content (as lines())", &lines)
            .field("warc_headers", &headers_pp)
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use warc::{Record, WarcHeader};

    use super::{Document, Metadata};

    #[test]
    fn test_from_record() {
        let record = Record::default();
        let body = "foo
        bar
        baz";

        let record = record.add_body(body);
        let metadata = Metadata::default();
        let doc = Document::from_record(record.clone(), metadata);

        let (headers, body) = record.into_raw_parts();
        assert_eq!(doc.content(), &String::from_utf8_lossy(&body).into_owned());
        assert_eq!(doc.warc_headers(), &headers.headers);
        assert_eq!(
            doc.warc_id(),
            String::from_utf8_lossy(headers.headers.get(&WarcHeader::RecordID).unwrap())
                .into_owned()
        );
    }

    #[test]
    fn test_serialize() {
        let m = Metadata::default();

        let serialized = serde_json::to_string_pretty(&m).unwrap();

        println!("{}", serialized);

        let m2: Metadata = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", m2);
    }
}
