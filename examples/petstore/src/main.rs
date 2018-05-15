#[macro_use]
extern crate finchers;
#[macro_use]
extern crate serde;
extern crate chrono;
extern crate futures;

mod model {
    use chrono::{DateTime, Utc};

    #[derive(Debug, Deserialize)]
    pub struct Category {
        id: u64,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Order {
        id: u64,
        pet_id: u64,
        quantity: u32,
        ship_date: DateTime<Utc>,
        status: String,
        complete: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct Pet {
        id: u64,
        category: Category,
        name: String,
        photo_urls: Vec<String>,
        tags: Vec<Tag>,
        status: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Tag {
        id: u64,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct User {
        id: u64,
        username: String,
        first_name: String,
        last_name: String,
        email: String,
        password: String,
        phone: String,
        user_status: u32,
    }
}

mod api {
    use finchers::endpoint::just;
    use finchers::endpoint::prelude::*;
    use finchers::{Json, Never};
    use futures::future;
    use model::*;

    pub fn endpoint() -> impl Endpoint<Output = &'static str> + 'static {
        let pet_id = param::<u64>().unwrap_ok();
        let pet_body = body::<Json<Pet>>().map_ok(Json::into_inner).unwrap_ok();

        let pet_endpoint = path("pet").right(choice![
            post(pet_body).map(|_new_pet| "add_pet"),
            delete(pet_id).map(|_id| "delete_pet"),
            get(path("findByStatus")).right(just("find_pets_by_status")),
            get(path("findByTags")).right(just("find_pets_by_tags")),
            get(pet_id).map(|_id| "get_pet_by_id"),
            put(pet_body).map(|_pet| "update_pet"),
            post(raw_body())
                .map_async(|_raw_body| future::ok::<_, Never>("update_pet_with_form"))
                .unwrap_ok(),
        ]);

        let order_id = param::<u64>().unwrap_ok();
        let order_body = body::<Json<Order>>().unwrap_ok();

        let store_endpoint = path("store").right(choice![
            delete(order_id).map(|_id| "delete_order"),
            get(path("inventory")).right(just("get_inventory")),
            get(path("order").right(order_id)).map(|_id| "get_order_by_id"),
            post(path("order").right(order_body)).right(just("place_order")),
        ]);

        let user_endpoint = path("user").right(just(""));

        path("v2").right(choice!(pet_endpoint, store_endpoint, user_endpoint))
    }
}

fn main() {
    finchers::run(api::endpoint());
}
