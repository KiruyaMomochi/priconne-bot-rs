# Event sync

同步不同版本間的活動資料。
Sync event data between different servers.

## Sync from post

It's hard. For example, the same gacha in different servers always has different names.
"蘭法" in Taiwan is "ランファ" in Japan, unless we have a mapping between them, which requires either manual work, machine learning, or a database. But if we have a database, we can just get event from it.

I do want to sync ALL posts between TW and JP, but I'm still figuring out how to do that.

## Sync from database

### Gacha

When a new gacha is added, a new item will show in table `gacha_data`.
Generally, `gacha_id` is same and unique across all servers. There may also be a `pick_up_chara_text` field, containing the name of the character that will be picked up.

Another table, `gacha_exchange_lineup`, contains the exchange lineup of the gacha.
`exahange_id` can be calculated by `gacha_id % 1000`, then we can find unit id.
