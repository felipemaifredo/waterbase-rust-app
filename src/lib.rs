//Libs
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

//Types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    pub documents: HashMap<String, Document>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Database {
    pub collections: HashMap<String, Collection>,
    pub storage_path: Option<String>,
}

/// Banco de dados compartilhado com lock granular por Collection.
/// Cada Collection possui seu próprio RwLock, permitindo leituras
/// paralelas em collections distintas e eliminando o bloqueio global.
#[derive(Debug)]
pub struct SharedDb {
    pub collections: RwLock<HashMap<String, Arc<RwLock<Collection>>>>,
    pub storage_path: Option<String>,
    pub sessions: RwLock<HashMap<String, String>>,
}

// Query Types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum WhereOp {
    #[serde(rename = "==")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhereFilter {
    pub field: String,
    pub op: WhereOp,
    pub value: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Query {
    pub r#where: Option<Vec<WhereFilter>>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

//Funcs
impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Boolean(b) => serializer.serialize_bool(*b),
            Value::Number(n) => serializer.serialize_f64(*n),
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(arr) => arr.serialize(serializer),
            Value::Object(obj) => obj.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Value::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Value::Number(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Value::Number(value as f64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Value::Number(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Value::String(value))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Value::Null)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    arr.push(elem);
                }
                Ok(Value::Array(arr))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut obj = HashMap::new();
                while let Some((key, val)) = map.next_entry()? {
                    obj.insert(key, val);
                }
                Ok(Value::Object(obj))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl serde::Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.fields.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fields = HashMap::<String, Value>::deserialize(deserializer)?;
        Ok(Document { fields })
    }
}

impl serde::Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.documents.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Collection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let documents = HashMap::<String, Document>::deserialize(deserializer)?;
        Ok(Collection { documents })
    }
}

impl serde::Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.collections.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Database {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let collections = HashMap::<String, Collection>::deserialize(deserializer)?;
        Ok(Database {
            collections,
            storage_path: None,
        })
    }
}

// Comparison Helpers
pub fn compare_for_sort(a: &Value, b: &Value) -> std::cmp::Ordering {
    let type_order = |v: &Value| -> u8 {
        match v {
            Value::Null => 0,
            Value::Boolean(_) => 1,
            Value::Number(_) => 2,
            Value::String(_) => 3,
            Value::Array(_) => 4,
            Value::Object(_) => 5,
        }
    };

    let ta = type_order(a);
    let tb = type_order(b);

    if ta != tb {
        return ta.cmp(&tb);
    }

    match (a, b) {
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Boolean(x), Value::Boolean(y)) => x.cmp(y),
        (Value::Number(x), Value::Number(y)) => {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(x), Value::String(y)) => x.cmp(y),
        (Value::Array(x), Value::Array(y)) => x.len().cmp(&y.len()),
        (Value::Object(x), Value::Object(y)) => x.len().cmp(&y.len()),
        _ => std::cmp::Ordering::Equal,
    }
}

pub fn compare_values(a: &Value, op: &WhereOp, b: &Value) -> bool {
    match op {
        WhereOp::Equal => a == b,
        WhereOp::NotEqual => a != b,
        WhereOp::GreaterThan => compare_for_sort(a, b) == std::cmp::Ordering::Greater,
        WhereOp::GreaterThanOrEqual => {
            let cmp = compare_for_sort(a, b);
            cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
        }
        WhereOp::LessThan => compare_for_sort(a, b) == std::cmp::Ordering::Less,
        WhereOp::LessThanOrEqual => {
            let cmp = compare_for_sort(a, b);
            cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
        }
    }
}

impl Document {
    pub fn new(fields: HashMap<String, Value>) -> Self {
        Document { fields }
    }
}

impl Collection {
    pub fn new() -> Self {
        Collection {
            documents: HashMap::new(),
        }
    }
}

impl Database {
    pub fn new() -> Self {
        Database {
            collections: HashMap::new(),
            storage_path: None,
        }
    }

    pub fn new_with_storage(path: String) -> Result<Self, String> {
        let mut collections = HashMap::new();
        let base_path = Path::new(&path);

        if !base_path.exists() {
            fs::create_dir_all(base_path).map_err(|e| e.to_string())?;
        } else {
            for entry in fs::read_dir(base_path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path_buf = entry.path();
                if path_buf.is_dir() {
                    let col_name = path_buf.file_name().ok_or("Invalid dir name")?.to_string_lossy().to_string();
                    let mut documents = HashMap::new();
                    
                    for file_entry in fs::read_dir(&path_buf).map_err(|e| e.to_string())? {
                        let file_entry = file_entry.map_err(|e| e.to_string())?;
                        let file_path = file_entry.path();
                        if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("bin") {
                            let doc_id = file_path.file_stem().ok_or("Invalid file name")?.to_string_lossy().to_string();
                            let bytes = fs::read(&file_path).map_err(|e| e.to_string())?;
                            let doc: Document = rmp_serde::from_slice(&bytes).map_err(|e| e.to_string())?;
                            documents.insert(doc_id, doc);
                        }
                    }
                    collections.insert(col_name, Collection { documents });
                }
            }
        }

        Ok(Database {
            collections,
            storage_path: Some(path),
        })
    }

    fn sync_document(&self, collection: &str, doc_id: &str) -> Result<(), String> {
        if let Some(ref base_path) = self.storage_path {
            if let Some(doc) = self.get_document(collection, doc_id) {
                let dir_path = format!("{}/{}", base_path, collection);
                if !Path::new(&dir_path).exists() {
                    fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;
                }
                let file_path = format!("{}/{}.bin", dir_path, doc_id);
                let bytes = rmp_serde::to_vec(doc).map_err(|e| e.to_string())?;
                fs::write(file_path, bytes).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn unsync_document(&self, collection: &str, doc_id: &str) -> Result<(), String> {
        if let Some(ref base_path) = self.storage_path {
            let file_path = format!("{}/{}/{}.bin", base_path, collection, doc_id);
            if Path::new(&file_path).exists() {
                fs::remove_file(file_path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn create_collection(&mut self, name: String) {
        self.collections.entry(name.clone()).or_insert_with(Collection::new);
        if let Some(ref base_path) = self.storage_path {
            let dir_path = format!("{}/{}", base_path, name);
            let _ = fs::create_dir_all(dir_path);
        }
    }

    pub fn delete_collection(&mut self, name: &str) -> Result<(), String> {
        self.collections.remove(name);
        if let Some(ref base_path) = self.storage_path {
            let dir_path = format!("{}/{}", base_path, name);
            if Path::new(&dir_path).exists() {
                fs::remove_dir_all(dir_path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn get_collection(&self, name: &str) -> Option<&Collection> {
        self.collections.get(name)
    }

    pub fn create_document(&mut self, collection_name: &str, doc_id: String, document: Document) -> Result<(), String> {
        if let Some(collection) = self.collections.get_mut(collection_name) {
            collection.documents.insert(doc_id.clone(), document);
        } else {
            let mut collection = Collection::new();
            collection.documents.insert(doc_id.clone(), document);
            self.collections.insert(collection_name.to_string(), collection);
        }
        self.sync_document(collection_name, &doc_id)?;
        Ok(())
    }

    pub fn get_document(&self, collection_name: &str, doc_id: &str) -> Option<&Document> {
        self.collections.get(collection_name)?.documents.get(doc_id)
    }

    pub fn update_document(&mut self, collection_name: &str, doc_id: &str, fields: HashMap<String, Value>) -> Result<(), String> {
        if let Some(collection) = self.collections.get_mut(collection_name) {
            if let Some(document) = collection.documents.get_mut(doc_id) {
                for (key, val) in fields {
                    document.fields.insert(key, val);
                }
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as f64;
                document.fields.insert("_updated_at".to_string(), Value::Number(now_ms));
            } else {
                return Err(format!("Document '{}' not found", doc_id));
            }
        } else {
            return Err(format!("Collection '{}' not found", collection_name));
        }
        self.sync_document(collection_name, doc_id)?;
        Ok(())
    }

    pub fn delete_document(&mut self, collection_name: &str, doc_id: &str) -> Result<(), String> {
        if let Some(collection) = self.collections.get_mut(collection_name) {
            if collection.documents.remove(doc_id).is_some() {
                self.unsync_document(collection_name, doc_id)?;
                Ok(())
            } else {
                Err(format!("Document '{}' not found", doc_id))
            }
        } else {
            Err(format!("Collection '{}' not found", collection_name))
        }
    }

    pub fn list_documents(&self, collection_name: &str) -> Result<Vec<(String, Document)>, String> {
        if let Some(collection) = self.collections.get(collection_name) {
            let mut docs: Vec<(String, Document)> = collection.documents.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            docs.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(docs)
        } else {
            Err(format!("Collection '{}' not found", collection_name))
        }
    }

    pub fn execute_query(&self, collection_name: &str, query: Query) -> Result<Vec<(String, Document)>, String> {
        let collection = self.get_collection(collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;

        let mut docs: Vec<(String, Document)> = collection.documents.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // 1. Filtrar
        if let Some(ref filters) = query.r#where {
            for filter in filters {
                docs.retain(|(_, doc)| {
                    if let Some(doc_val) = doc.fields.get(&filter.field) {
                        compare_values(doc_val, &filter.op, &filter.value)
                    } else {
                        false
                    }
                });
            }
        }

        // 2. Ordenar
        if let Some(ref order) = query.order_by {
            docs.sort_by(|a, b| {
                let val_a = a.1.fields.get(&order.field).unwrap_or(&Value::Null);
                let val_b = b.1.fields.get(&order.field).unwrap_or(&Value::Null);
                
                let cmp = compare_for_sort(val_a, val_b);
                match order.direction {
                    OrderDirection::Ascending => cmp,
                    OrderDirection::Descending => cmp.reverse(),
                }
            });
        } else {
            docs.sort_by(|a, b| a.0.cmp(&b.0));
        }

        // 3. Offset
        if let Some(off) = query.offset {
            if off < docs.len() {
                docs.drain(0..off);
            } else {
                docs.clear();
            }
        }

        // 4. Limitar
        if let Some(lim) = query.limit {
            docs.truncate(lim);
        }

        Ok(docs)
    }
}

impl SharedDb {
    /// Converte um `Database` carregado do disco para `SharedDb`,
    /// envolvendo cada Collection em seu próprio `Arc<RwLock<>>`.
    pub fn from_database(db: Database) -> Self {
        let mut cols = HashMap::new();
        for (name, col) in db.collections {
            cols.insert(name, Arc::new(RwLock::new(col)));
        }
        SharedDb {
            collections: RwLock::new(cols),
            storage_path: db.storage_path,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    async fn sync_document(&self, collection: &str, doc_id: &str, doc: &Document) -> Result<(), String> {
        if let Some(ref base_path) = self.storage_path {
            let dir_path = format!("{}/{}", base_path, collection);
            let file_path = format!("{}/{}.bin", dir_path, doc_id);
            let bytes = rmp_serde::to_vec(doc).map_err(|e| e.to_string())?;
            tokio::task::spawn_blocking(move || {
                fs::create_dir_all(&dir_path).map_err(|e| e.to_string())?;
                fs::write(&file_path, &bytes).map_err(|e| e.to_string())
            }).await.map_err(|e| e.to_string())??;
        }
        Ok(())
    }

    async fn unsync_document(&self, collection: &str, doc_id: &str) -> Result<(), String> {
        if let Some(ref base_path) = self.storage_path {
            let file_path = format!("{}/{}/{}.bin", base_path, collection, doc_id);
            tokio::task::spawn_blocking(move || {
                if Path::new(&file_path).exists() {
                    fs::remove_file(&file_path).map_err(|e| e.to_string())?;
                }
                Ok::<(), String>(())
            }).await.map_err(|e| e.to_string())??;
        }
        Ok(())
    }

    /// Lista os nomes de todas as collections, ordenados alfabeticamente.
    pub async fn list_collections(&self) -> Vec<String> {
        let cols = self.collections.read().await;
        let mut names: Vec<String> = cols.keys().cloned().collect();
        names.sort();
        names
    }

    /// Cria uma nova collection (idempotente — não duplica se já existir).
    pub async fn create_collection(&self, name: String) {
        let mut cols = self.collections.write().await;
        cols.entry(name.clone())
            .or_insert_with(|| Arc::new(RwLock::new(Collection::new())));
        if let Some(ref base_path) = self.storage_path {
            let dir_path = format!("{}/{}", base_path, name);
            let _ = fs::create_dir_all(dir_path);
        }
    }

    /// Exclui permanentemente uma collection do mapa em memória e do disco.
    pub async fn delete_collection(&self, name: &str) -> Result<(), String> {
        let mut cols = self.collections.write().await;
        cols.remove(name);
        if let Some(ref base_path) = self.storage_path {
            let dir_path = format!("{}/{}", base_path, name);
            if Path::new(&dir_path).exists() {
                fs::remove_dir_all(dir_path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// Cria ou substitui um documento em uma collection.
    /// Se a collection não existir, ela é criada automaticamente.
    pub async fn create_document(
        &self,
        collection_name: &str,
        doc_id: String,
        document: Document,
    ) -> Result<(), String> {
        if collection_name == "authentication" {
            for key in document.fields.keys() {
                if key != "email" && key != "password_hash" && !key.starts_with('_') {
                    return Err("A coleção 'authentication' só pode aceitar os campos 'email' e 'password_hash'".to_string());
                }
            }
        }
        // Tenta obter o Arc sem write-lock primeiro (caminho feliz)
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name).cloned()
        };

        let col_arc = match col_arc {
            Some(arc) => arc,
            None => {
                // Collection não existe — precisa de write-lock no mapa
                let mut cols = self.collections.write().await;
                cols.entry(collection_name.to_string())
                    .or_insert_with(|| Arc::new(RwLock::new(Collection::new())))
                    .clone()
            }
        };

        {
            let mut col = col_arc.write().await;
            col.documents.insert(doc_id.clone(), document.clone());
        }

        self.sync_document(collection_name, &doc_id, &document).await?;
        Ok(())
    }

    /// Retorna uma cópia do documento, se existir.
    pub async fn get_document(
        &self,
        collection_name: &str,
        doc_id: &str,
    ) -> Option<Document> {
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name)?.clone()
        };
        let col = col_arc.read().await;
        col.documents.get(doc_id).cloned()
    }

    /// Aplica um merge de campos sobre um documento existente.
    pub async fn update_document(
        &self,
        collection_name: &str,
        doc_id: &str,
        fields: HashMap<String, Value>,
    ) -> Result<(), String> {
        if collection_name == "authentication" {
            for key in fields.keys() {
                if key != "email" && key != "password_hash" && !key.starts_with('_') {
                    return Err("A coleção 'authentication' só pode aceitar os campos 'email' e 'password_hash'".to_string());
                }
            }
        }
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name)
                .cloned()
                .ok_or_else(|| format!("Collection '{}' not found", collection_name))?
        };

        let updated_doc = {
            let mut col = col_arc.write().await;
            let doc = col
                .documents
                .get_mut(doc_id)
                .ok_or_else(|| format!("Document '{}' not found", doc_id))?;
            for (k, v) in fields {
                doc.fields.insert(k, v);
            }
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64;
            doc.fields.insert("_updated_at".to_string(), Value::Number(now_ms));
            doc.clone()
        };

        self.sync_document(collection_name, doc_id, &updated_doc).await?;
        Ok(())
    }

    /// Remove um documento de uma collection.
    pub async fn delete_document(
        &self,
        collection_name: &str,
        doc_id: &str,
    ) -> Result<(), String> {
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name)
                .cloned()
                .ok_or_else(|| format!("Collection '{}' not found", collection_name))?
        };

        {
            let mut col = col_arc.write().await;
            if col.documents.remove(doc_id).is_none() {
                return Err(format!("Document '{}' not found", doc_id));
            }
        }

        self.unsync_document(collection_name, doc_id).await?;
        Ok(())
    }

    /// Lista todos os documentos de uma collection, ordenados pelo ID.
    pub async fn list_documents(
        &self,
        collection_name: &str,
    ) -> Result<Vec<(String, Document)>, String> {
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name)
                .cloned()
                .ok_or_else(|| format!("Collection '{}' not found", collection_name))?
        };

        let col = col_arc.read().await;
        let mut docs: Vec<(String, Document)> = col
            .documents
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        docs.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(docs)
    }

    /// Executa uma query com filtros, ordenação e limite.
    pub async fn execute_query(
        &self,
        collection_name: &str,
        query: Query,
    ) -> Result<Vec<(String, Document)>, String> {
        let col_arc = {
            let cols = self.collections.read().await;
            cols.get(collection_name)
                .cloned()
                .ok_or_else(|| format!("Collection '{}' not found", collection_name))?
        };

        let mut docs: Vec<(String, Document)> = {
            let col = col_arc.read().await;
            col.documents
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        if let Some(ref filters) = query.r#where {
            for filter in filters {
                docs.retain(|(_, doc)| {
                    if let Some(doc_val) = doc.fields.get(&filter.field) {
                        compare_values(doc_val, &filter.op, &filter.value)
                    } else {
                        false
                    }
                });
            }
        }

        if let Some(ref order) = query.order_by {
            docs.sort_by(|a, b| {
                let val_a = a.1.fields.get(&order.field).unwrap_or(&Value::Null);
                let val_b = b.1.fields.get(&order.field).unwrap_or(&Value::Null);
                let cmp = compare_for_sort(val_a, val_b);
                match order.direction {
                    OrderDirection::Ascending => cmp,
                    OrderDirection::Descending => cmp.reverse(),
                }
            });
        } else {
            docs.sort_by(|a, b| a.0.cmp(&b.0));
        }

        if let Some(off) = query.offset {
            if off < docs.len() {
                docs.drain(0..off);
            } else {
                docs.clear();
            }
        }

        if let Some(lim) = query.limit {
            docs.truncate(lim);
        }

        Ok(docs)
    }
}

//Main
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_composition() {
        let mut fields = HashMap::new();
        fields.insert(String::from("nome"), Value::String(String::from("Felipe")));
        fields.insert(String::from("idade"), Value::Number(29.0));
        fields.insert(String::from("ativo"), Value::Boolean(true));
        fields.insert(String::from("nulo"), Value::Null);

        let doc = Document { fields };

        let mut documents = HashMap::new();
        documents.insert(String::from("felipe_id"), doc);

        let collection = Collection { documents };

        let mut collections = HashMap::new();
        collections.insert(String::from("users"), collection);

        let db = Database { collections, storage_path: None };

        let user_col = db.collections.get("users").unwrap();
        let user_doc = user_col.documents.get("felipe_id").unwrap();

        assert_eq!(user_doc.fields.get("nome"), Some(&Value::String(String::from("Felipe"))));
        assert_eq!(user_doc.fields.get("idade"), Some(&Value::Number(29.0)));
        assert_eq!(user_doc.fields.get("ativo"), Some(&Value::Boolean(true)));
        assert_eq!(user_doc.fields.get("nulo"), Some(&Value::Null));
    }

    #[test]
    fn test_crud_operations() {
        let mut db = Database::new();
        db.create_collection("products".to_string());

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("Laptop".to_string()));
        fields.insert("price".to_string(), Value::Number(999.99));
        let doc = Document::new(fields);

        assert!(db.create_document("products", "prod_1".to_string(), doc).is_ok());

        // Get
        let retrieved = db.get_document("products", "prod_1").unwrap();
        assert_eq!(retrieved.fields.get("name"), Some(&Value::String("Laptop".to_string())));

        // Update
        let mut updates = HashMap::new();
        updates.insert("price".to_string(), Value::Number(899.99));
        updates.insert("in_stock".to_string(), Value::Boolean(true));
        assert!(db.update_document("products", "prod_1", updates).is_ok());

        let updated = db.get_document("products", "prod_1").unwrap();
        assert_eq!(updated.fields.get("price"), Some(&Value::Number(899.99)));
        assert_eq!(updated.fields.get("in_stock"), Some(&Value::Boolean(true)));

        // List
        let list = db.list_documents("products").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "prod_1");

        // Delete
        assert!(db.delete_document("products", "prod_1").is_ok());
        assert!(db.get_document("products", "prod_1").is_none());
    }

    #[test]
    fn test_msgpack_serialization() {
        let mut fields = HashMap::new();
        fields.insert("nome".to_string(), Value::String("Felipe".to_string()));
        fields.insert("idade".to_string(), Value::Number(29.0));
        let doc = Document::new(fields);

        let bytes = rmp_serde::to_vec(&doc).unwrap();
        assert!(!bytes.is_empty());

        let deserialized: Document = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(deserialized.fields.get("nome"), Some(&Value::String("Felipe".to_string())));
        assert_eq!(deserialized.fields.get("idade"), Some(&Value::Number(29.0)));
    }

    #[test]
    fn test_disk_persistence() {
        let temp_dir_path = "temp_test_db_data".to_string();
        
        if Path::new(&temp_dir_path).exists() {
            let _ = fs::remove_dir_all(&temp_dir_path);
        }

        let mut db = Database::new_with_storage(temp_dir_path.clone()).unwrap();
        db.create_collection("users".to_string());
        
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("John".to_string()));
        let doc = Document::new(fields);
        db.create_document("users", "john_id".to_string(), doc).unwrap();

        let file_path = format!("{}/users/john_id.bin", temp_dir_path);
        assert!(Path::new(&file_path).exists());

        let db2 = Database::new_with_storage(temp_dir_path.clone()).unwrap();
        
        let retrieved = db2.get_document("users", "john_id").unwrap();
        assert_eq!(retrieved.fields.get("name"), Some(&Value::String("John".to_string())));

        let mut db3 = db2;
        db3.delete_document("users", "john_id").unwrap();
        assert!(!Path::new(&file_path).exists()); // arquivo .bin deve ter sido removido

        let _ = fs::remove_dir_all(&temp_dir_path);
    }

    #[test]
    fn test_querying() {
        let mut db = Database::new();
        db.create_collection("users".to_string());

        // Document 1
        let mut f1 = HashMap::new();
        f1.insert("name".to_string(), Value::String("Ana".to_string()));
        f1.insert("age".to_string(), Value::Number(20.0));
        f1.insert("active".to_string(), Value::Boolean(true));
        db.create_document("users", "u1".to_string(), Document::new(f1)).unwrap();

        // Document 2
        let mut f2 = HashMap::new();
        f2.insert("name".to_string(), Value::String("Carlos".to_string()));
        f2.insert("age".to_string(), Value::Number(30.0));
        f2.insert("active".to_string(), Value::Boolean(true));
        db.create_document("users", "u2".to_string(), Document::new(f2)).unwrap();

        // Document 3
        let mut f3 = HashMap::new();
        f3.insert("name".to_string(), Value::String("Beto".to_string()));
        f3.insert("age".to_string(), Value::Number(25.0));
        f3.insert("active".to_string(), Value::Boolean(false));
        db.create_document("users", "u3".to_string(), Document::new(f3)).unwrap();

        // Query 1: Filter where active == true
        let q1 = Query {
            r#where: Some(vec![WhereFilter {
                field: "active".to_string(),
                op: WhereOp::Equal,
                value: Value::Boolean(true),
            }]),
            order_by: None,
            limit: None,
            offset: None,
        };
        let res1 = db.execute_query("users", q1).unwrap();
        assert_eq!(res1.len(), 2);
        assert_eq!(res1[0].0, "u1"); // sorted by default ID (u1, u2)
        assert_eq!(res1[1].0, "u2");

        // Query 2: Filter where age > 22, sort by age desc
        let q2 = Query {
            r#where: Some(vec![WhereFilter {
                field: "age".to_string(),
                op: WhereOp::GreaterThan,
                value: Value::Number(22.0),
            }]),
            order_by: Some(OrderBy {
                field: "age".to_string(),
                direction: OrderDirection::Descending,
            }),
            limit: None,
            offset: None,
        };
        let res2 = db.execute_query("users", q2).unwrap();
        assert_eq!(res2.len(), 2);
        assert_eq!(res2[0].0, "u2"); // Carlos (30)
        assert_eq!(res2[1].0, "u3"); // Beto (25)

        // Query 3: Filter where age > 18, limit 1, sort age asc
        let q3 = Query {
            r#where: Some(vec![WhereFilter {
                field: "age".to_string(),
                op: WhereOp::GreaterThan,
                value: Value::Number(18.0),
            }]),
            order_by: Some(OrderBy {
                field: "age".to_string(),
                direction: OrderDirection::Ascending,
            }),
            limit: Some(1),
            offset: None,
        };
        let res3 = db.execute_query("users", q3).unwrap();
        assert_eq!(res3.len(), 1);
        assert_eq!(res3[0].0, "u1"); // Ana (20)
    }

    #[test]
    fn test_delete_collection() {
        let temp_dir_path = "temp_test_db_delete_col".to_string();
        if Path::new(&temp_dir_path).exists() {
            let _ = fs::remove_dir_all(&temp_dir_path);
        }
        let mut db = Database::new_with_storage(temp_dir_path.clone()).unwrap();
        db.create_collection("temporary".to_string());
        
        let mut fields = HashMap::new();
        fields.insert("key".to_string(), Value::String("value".to_string()));
        let doc = Document::new(fields);
        db.create_document("temporary", "doc1".to_string(), doc).unwrap();

        let dir_path = format!("{}/temporary", temp_dir_path);
        assert!(Path::new(&dir_path).exists());

        // Deletar
        db.delete_collection("temporary").unwrap();
        assert!(!Path::new(&dir_path).exists());
        assert!(db.get_collection("temporary").is_none());

        let _ = fs::remove_dir_all(&temp_dir_path);
    }

    #[test]
    fn test_querying_offset() {
        let mut db = Database::new();
        db.create_collection("users".to_string());

        for i in 0..5 {
            let mut f = HashMap::new();
            f.insert("val".to_string(), Value::Number(i as f64));
            db.create_document("users", format!("u{}", i), Document::new(f)).unwrap();
        }

        // Query: offset 2, limit 2
        let q = Query {
            r#where: None,
            order_by: None,
            limit: Some(2),
            offset: Some(2),
        };
        let res = db.execute_query("users", q).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].0, "u2");
        assert_eq!(res[1].0, "u3");
    }

    #[test]
    fn test_automatic_updated_at() {
        let mut db = Database::new();
        db.create_collection("users".to_string());
        
        let mut fields = HashMap::new();
        fields.insert("nome".to_string(), Value::String("Original".to_string()));
        db.create_document("users", "u1".to_string(), Document::new(fields)).unwrap();

        let doc_before = db.get_document("users", "u1").unwrap();
        assert!(doc_before.fields.get("_updated_at").is_none());

        let mut updates = HashMap::new();
        updates.insert("nome".to_string(), Value::String("Updated".to_string()));
        db.update_document("users", "u1", updates).unwrap();

        let doc_after = db.get_document("users", "u1").unwrap();
        assert!(doc_after.fields.get("_updated_at").is_some());
        if let Some(Value::Number(ts)) = doc_after.fields.get("_updated_at") {
            assert!(*ts > 0.0);
        } else {
            panic!("_updated_at should be a Number");
        }
    }
}
