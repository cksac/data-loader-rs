use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema};
use async_std::task;
use async_trait::async_trait;
use dataloader::eager::cached::Loader;
use dataloader::BatchFn;
use fake::faker::company::en::CompanyName;
use fake::faker::name::en::Name;
use fake::{Dummy, Fake, Faker};
use std::collections::HashMap;

pub struct CultBatcher;

#[async_trait]
impl BatchFn<i32, Cult> for CultBatcher {
    type Error = ();

    async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<Cult, Self::Error>> {
        println!("load cult by batch {:?}", keys);
        let ret = keys
            .iter()
            .map(|k| {
                let mut cult: Cult = Faker.fake();
                cult.id = k.clone();
                (k.clone(), Ok(cult))
            })
            .collect();
        ret
    }
}

#[derive(Clone)]
pub struct AppContext {
    cult_loader: Loader<i32, Cult, (), CultBatcher>,
}

impl AppContext {
    pub fn new() -> AppContext {
        AppContext {
            cult_loader: Loader::new(CultBatcher),
        }
    }
}

struct Query;

#[async_graphql::Object]
impl Query {
    #[field]
    async fn persons(&self, _ctx: &Context<'_>) -> Vec<Person> {
        let persons = fake::vec![Person; 10..20];
        println!("get persons");
        persons
    }
}

#[derive(Debug, Clone, Dummy)]
pub struct Person {
    #[dummy(faker = "1..999")]
    pub id: i32,
    #[dummy(faker = "Name()")]
    pub name: String,
    #[dummy(faker = "1..999")]
    pub cult: i32,
}

#[async_graphql::Object]
impl Person {
    #[field]
    async fn id(&self) -> i32 {
        self.id
    }

    #[field]
    async fn name(&self) -> &str {
        self.name.as_str()
    }

    #[field]
    async fn cult(&self, ctx: &Context<'_>) -> Cult {
        println!("load Person[{}].cult[{}]", self.id, self.cult);
        let fut = ctx.data::<AppContext>().cult_loader.load(self.cult);
        fut.await.expect("get cult")
    }
}

#[derive(Debug, Clone, Dummy)]
pub struct Cult {
    #[dummy(faker = "1..999")]
    pub id: i32,
    #[dummy(faker = "CompanyName()")]
    pub name: String,
}

#[async_graphql::Object]
impl Cult {
    #[field]
    async fn id(&self) -> i32 {
        self.id
    }

    #[field]
    async fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn main() {
    env_logger::init();

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription).data(AppContext::new());
    let q = "query {
        persons {
          id
          name
          cult {
            id
            name
          }
        }
      }";
    let f = schema.query(&q).execute();
    let _r = task::block_on(f).unwrap();
}