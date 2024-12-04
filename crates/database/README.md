# Database crate

A lightweight library for managing MongoDB operations. This library provides an interface, the `Repository` trait with default implementations for interacting with MongoDB collections. It is used by all the plugins in the workspace that require database access.

## Usage

### Requirements

* [MongoDB](https://www.mongodb.com) server instance
* Environment variables:
  * `MONGO_URI`: MongoDB connection string
  * `MONGO_DB`: Database name

### Example

* Define an entity

```rust
use database::Repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
}

impl Identifiable for MyEntity {
    fn id(&self) -> Option<ObjectId> {
        self.id.clone()
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}
```

* Implement the `Repository` trait(the only required method is `get_collection`)

```rust
struct MyEntityRepository {
    collection: Arc<RwLock<Collection<MyEntity>>>,
}

#[async_trait]
impl Repository<MyEntity> for MyEntityRepository {
    fn get_collection(&self) -> Arc<RwLock<Collection<MyEntity>>> {
        self.collection.clone()
    }
}
```

* Use the repository

```rust
let db = get_or_init_database();
let repo = MyEntityRepository {
    collection: Arc::new(RwLock::new(db.read().await.collection("my_entities"))),
};
let entity = MyEntity { id: None, name: "example".to_string() };
repo.store(entity).await?;
```
