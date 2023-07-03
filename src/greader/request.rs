use chrono::NaiveDateTime;

pub enum StreamRanking {
    NewestFirst,
    OldestFirst,
}

pub enum StreamState {
    Read,
    KeptUnread,
    ReadingList,
    Starred,
}

pub enum StreamTag {
    Label(Option<String>, String),
    State(Option<String>, StreamState),
}

pub enum StreamId {
    Feed(String),
    Tag(StreamTag),
}

pub enum RequestType {
    Token,
    UserInfo,
    UnreadCount,
    SubscriptionList,
    StreamContents {
        stream_id: StreamId,
        ranking: StreamRanking,
        number: u32,
        continuation: String,
        exclude: Option<StreamTag>,
        exclude_older_than: Option<NaiveDateTime>,
        exclude_newer_than: Option<NaiveDateTime>,
    },
    StreamItemsIds {
        stream_id: StreamId,
        ranking: StreamRanking,
        number: u32,
        continuation: String,
        exclude: Option<StreamTag>,
        exclude_older_than: Option<NaiveDateTime>,
        exclude_newer_than: Option<NaiveDateTime>,
    },
    StreamItemsCount(StreamId),
    StreamItemsContents(Vec<String>),
    TagList,
    EditTag {
        item_ids: Vec<String>,
        add_tags: Vec<StreamTag>,
        remove_tags: Vec<StreamTag>,
    },
    MarkAllAsRead {
        stream_id: StreamId,
        older_than: Option<NaiveDateTime>,
    },
}
