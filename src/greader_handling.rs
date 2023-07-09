use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::data;
use crate::error::Error;
use crate::greader::request::*;
use crate::greader::response::*;
use crate::models::feed::Feed as DbFeed;
use crate::models::group::Group as DbGroup;

type DataResult<T> = Result<T, Error<diesel::result::Error>>;

fn format_tag(group: DbGroup) -> Tag {
    Tag {
        id: StreamTag::Label(None, group.id.to_string()),
        sort_id: group.title,
    }
}

fn format_subscription(feed: DbFeed, group: Option<DbGroup>) -> Subscription {
    Subscription {
        title: feed.title,
        first_item_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        html_url: feed.site_url.unwrap(),
        sort_id: "".to_owned(),
        id: StreamId::Feed(feed.id.to_string()),
        categories: group.map(|group| {
            SubscriptionCategory {
                id: StreamTag::Label(None, group.id.to_string()),
                label: group.title,
            }
        }).into_iter().collect(),
    }
}

fn load_labels(conn: &mut PgConnection) -> DataResult<Vec<Tag>> {
    let labels = data::load_groups(conn)
        .map_err(fill_err!("Error loading groups"))?
        .into_iter()
        .map(format_tag)
        .collect();
    Ok(labels)
}

fn load_subscriptions(conn: &mut PgConnection) -> DataResult<Vec<Subscription>> {
    use crate::schema::{feed, feed_group};

    let subs = feed::table
        .left_join(feed_group::table)
        .select((DbFeed::as_select(), Option::<DbGroup>::as_select()))
        .load::<(DbFeed, Option<DbGroup>)>(conn)
        .map_err(fill_err!("Error loading feeds"))?
        .into_iter()
        .map(|(f, g)| format_subscription(f, g))
        .collect();
    Ok(subs)
}

pub fn handle_api_request(
    request: &RequestType,
    conn: &mut PgConnection,
) -> DataResult<Response> {
    use RequestType::*;

    let response: Response = match request {
        UserInfo => UserInfoResponse {
            user_id: "123".to_owned(),
            user_name: "Name".to_owned(),
            user_profile_id: "123".to_owned(),
            user_email: "username@gmail.com".to_owned(),
            is_blogger_user: true,
            signup_time: chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            public_user_name: "username".to_owned(),
        }.into(),
        TagList => TagListResponse {
            tags: load_labels(conn)?,
        }.into(),
        SubscriptionList => SubscriptionListResponse {
            subscriptions: load_subscriptions(conn)?,
        }.into(),
        _ => "OK".to_owned().into(),
    };

    Ok(response)
}
