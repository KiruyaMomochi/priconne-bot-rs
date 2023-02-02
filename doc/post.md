# TODO

Maybe we still can split checkxxx from streams?

# Post

機器人很重要的一個目標就是準確完整的記下所有的遊戲公告和新聞，統一稱之為「文章」(post)。

A post may come from:

- Announcement「公告」: from API server `[api]/information/`.
- News「最新消息」: from official website <www.princessconnect.so-net.tw/news/>.
- Facebook Page: <https://www.facebook.com/SonetPCR>.
- Bahamut GNN News sometimes, like before anniversary event.

Different kinds of post has different id, or may not have id at all,
we will put best effort to avoid sending duplicate post.

## Fetching

We fetch post by **pulling** the server periodically.
If a specific count of posts have all been fetched, we stop pulling.

## Same post check

### Different source

We consider two new posts from from different sources are the same if.

1. They have the same title after removing whitespace, square brackets and update label.
2. They are sent within a specific time period.

Although commonnly announce and news are sent in nearly same time, sometimes it fails as a wrong date is set, or other reasons. This can lead to false negative. When such a case happens, we delete the new post manually, and our bot should migrate the new post to the old one.

### Update post (same source)

We consider posts from single source as updated if:

1. Message has same ID but a new time or a new title.
2. Message has a different ID, but it's name contains update label.

The time change is hard for news, because it only has create time and only has date. The second condition is required as a new post may be sent in some days later, using a new ID, and only it's title contains the update label. We guess a possible updated post by comparing the title, but it maybe not 100% accurate.

### Update post (different source)

If a update comes from a different source, it maybe ignored when new update time is near to the old one.

An updated post has a new Post object. We send a new message to channel, replying to the old one.

Sometimes, user can't find what updated between the new and old post. We may need to manually update the message to tell user what's changed, where bot can give us some hints.

## History Checking

When I want to find a history post, I can use tag to search.
But if it's a updated post, we should tell user which is the old one.

## Posting

A posted Telegram message includes:

- Tags.
- Title without square brackets.
- An summary of events in post, or other important information when applicable.
- Telegraph link to the post.
- Created time.
- Sources.

Abstract information is generated by checking the section headers.
For a really short post without media, send full content instead, but we still upload it to Telegraph.

Once a post message is sent, user may further edit it.
Only the source part can be updated by bot.

## Editing

Message editing is hard to implement, since a message can be edited manually. When a long post is published, it's possible for us to add a summary in message body. If we still update message, the summary will be lost.

For this problem, either reply to the original message instead of editing it, or only update specific fields in the message. It's also possible to ask user upserting summary to our database. However, this approach agains our philosophy: user is central, tool is helper.

Considering that we should notify our user when a post is updated, it's better to reply to the original message, instead of editing it, but if we only see another source is found, we may want to update the message instead, as we doesn't care it.

## Telegraph

When uploading to Telegraph, several rules are applied.

- Post from news site may contain a lot of line breaks before content.
  We should remove them all.
- The news is still use HTTP without SSL, causing images failed to load using browser.
  We need to reupload the image to Telegraph.

## Model

Since we now considering split priconne service from Telegram, there may be no `message_id` field in Post. We also need to save all sent messages, each message corresponds to a post.

The problem is that we do need a way to make sure all posts are sent, and not sent twice. It's easy when these two services are combined.

### Methods to keep sync

We can check syncing status by the same method as service <-> api interaction. But that feels ugly.

Looks like mongodb has something built-in like <https://stackoverflow.com/questions/62712518/mongodb-how-to-get-data-not-exist-on-other-collection>, but we don't know its performance. Also, that still doesn't like what we are trying to solve.

But as long as we want to decouple them, we can't make sure a messgae is sent successfully before adding an item to database - actually we can do it using callback, but does this means we must wait for all things succeed before insertion? It's strange.

Actually it's debate between using **push** model or **pull** model. Our answer is a hybrid one:

- Use push model to make sure an message is sent asap, but it may fail if very bad thing happens.
- Use pull model as a fallback. If something can even solve using pulling, tell me that something goes wrong.

We can give time limit to pull model. Something like one day.

### Post

We use a `Post` model to store it.
Each `Post` object is mapped to one Telegram message.

```rust
pub struct Post {
    /// Post ID.
    /// Can generate by `bson::oid::ObjectId::new()`.
    #[serde(rename = "_id")]
    pub id: oid::ObjectId,
    /// The title of the post.
    pub title: String,
    /// Mapped title for matching.
    pub mapped_title: String,
    /// Region of the post.
    pub region: Region,
    /// Source of the post.
    pub sources: Vec<PostSource>,
    #[serde_as(as = "mongodb::bson::DateTime")]
    /// The time when the post was created.
    pub create_time: DateTime<chrono::Utc>,
    #[serde_as(as = "Option<mongodb::bson::DateTime>")]
    /// The time when the post was updated.
    pub update_time: Option<DateTime<chrono::Utc>>,
    /// History post ID.
    pub history: Option<oid::ObjectId>,
    /// Tags of the post.
    pub tags: LinkedHashSet<String>,
    /// Events contained in the post.
    pub events: Vec<EventPeriod>,
    /// Telegraph page URL.
    pub telegraph: String,
    /// Message ID in chat.
    pub message_id: Option<i32>,
}
```

For example, this is a serialized post

```json
{
  "_id": "5c8f8f8f8f8f8f8f8f8f8f8f",
  "title": "【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！",
  "mapped_title": "《公主祭典 獎勵轉蛋》★3「蘭法」期間限定角色登場！舉辦預告！",
  "region": "TW",
  "sources": [
    {
      "type": "Announce",
      "api": "PROD3",
      "id": 1803
    },
    {
      "type": "News",
      "id": 1774
    }
  ],
  "create_time": "2022-07-01T3:55:00",
  "update_time": null,
  "history": null,
  "tags" : [
    "公主祭典",
    "蘭法",
    "轉蛋",
    "獎勵轉蛋",
  ],
  "events": [
    {
      "start": "2022-07-02T8:00:00",
      "end": "2022-07-05T7:59:00",
      "title": "公主祭典 獎勵轉蛋"
    }
  ],
  "telegraph": "https://telegra.ph/轉蛋公主祭典-獎勵轉蛋3蘭法期間限定角色登場舉辦預告-07-01",
  "message_id": 1800
}
```

### Message

The title of a message is different from post title.
A message also needs summary which is generated from html content.
Therefore, we create a `Message` struct from post and html.
It will not save to database.

```rust
struct Message {
    /// Tags in the message.
    pub tags: LinkedHashSet<String>,
    /// The title in the message.
    pub title: String,
    /// The summary in the message.
    pub summary: Option<String>,
    /// Events in the message.
    pub events: Vec<EventPeriod>,
    /// Telegraph URL.
    pub telegraph: Option<String>,
    /// Source URL.
    pub source: Option<String>,
    /// Created time.
    pub create_time: DateTime<chrono::Utc>,
    /// Source of the message.
    pub sources: Vec<PostSource>,
    /// Silent mode.
    pub silent: bool,
    /// Message ID to reply to.
    pub reply_to: Option<i32>,
}
```

## Implementation

Without loss of generality, we take news as the source of the post.
Some scheduler periodically call `check_new_news` function of a struct.
This function get a stream from priconne service, for `News` object in stream:

1. Check if it's there is a news with same ID in `news` db collection.
2. If no, insert it to the collection.
3. If yes, and that news is older, update it.
4. If yes, and that news is same, do nothing.
5. Check if there is an existing post in `posts` collection that:
   - contains a news item with same ID, or
   - doesn't contains news item, but contains an item
     - with same mapped title, and
     - sent within 24 hours
6. If such a post is found, whose update time is newer, we add item
7. If such a post is found, whose update time is older and
   - doesn't contains news item, then add item but not update time
   - contains news item, then update update time
8. If no such post is found, then create a new post.

Input: Resource<>, IsNew, IsUpdated
Output: Operation of Post

| Resource | Title | 24Hours | Operation | Note |
| ------------ | --------------- | -------------------- | -------------- | --- |
| Same | Same | - | None | - | - |
| Same | Diff | - | Update | Same | Only send if we're sure it's updated |
| Other | Same | Yes | Update | - |
| Conflict | Same | Yes | Report | - |

## Story

Now So-net published a new post, we assume it appears in both in-game api server and website, then updated the post.
Our goal is to fetch the post, send it to Telegram channel, and show the update correctly even if the post is edited.

As soon as the post is published, we find the post by pulling API server and

1. Checking existing posts tells us this post is new, so we need to create a new one.
2. Fetch the page of post, insert page to database, and create a Telegraph page.
3. At the same time, we extract information like events, tags, etc.
4. Create a `Post` object containing all these information, and save it to database.
5. Convert it to a Telegram message, sent it to channel and upsert message id.

Because the message is long, we manually update the message to include an summary.

We also found the post in website, so

1. Checking existing posts tells us there is already a post with this title, but from another source.
2. Insert website page to database, and upsert the new source to the post.
3. Since message_id is not `None`, this message has already been sent.
4. Update the message by replacing old source part with the new one.

Some times later, the post of same id is updated.

1. Detected the update in API as update time in the page has changed.
2. Further check that page update time is newser than post update time.
3. Create a new post with the new update time. (?)

## Future Work

Add Facebook as source too. But facebook post has limitation of length and format, so it may be hard to implement.
However, Facebook post contains special message during birthday or anniversary, so we still need to consider it.

Another source of post is Bahamut GNN, since they provide a RSS feed, it may be easier, but their post is hard to convert to Telegraph page.