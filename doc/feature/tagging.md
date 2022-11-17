# Tagging

This article describes our tagging system.

## Motivation

Proper tags facilitate reader to quickly find the content they are looking for, and search for similar content.

## Tag Generation

Currently, we generate tags from title of content, from source of content, user-provided rules and from title itself. We collect all these tags and ensure final list is unique.

### From content source

Some content sources may have categories, and we can use them as tags. 
In information page, we have icons, like "活動", "轉蛋", "更新", etc.
In news page, we have "活動", "系統", etc.

### From user-provided rules

User provides us a list of tag rules, in the form of a mapping between tag and regex. We then match the title against the regex to generate the tag.

A static list of tags are not sufficient. Other words like character name, event name, etc. should also be included. Since these information are mostly in quotation marks or brackets of title, we get them there.

### From title itself

We can extract other tags by matching quotes or brackets.
In common, title begin with a square bracket, containing a tag.
There may also `「」` or `《》`.
Nested quotes like parenthesis in `「可可蘿（祭服）」` will be ignored.
When text in quotation contains spaces or punctuation, we will split it and only tag the first token.

### Example

- 【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法（新年）」期間限定角色登場！舉辦預告！

  - Tags from rules: `轉蛋`, `公主祭典`, `獎勵轉蛋`
  - Tags from title: `轉蛋`, `公主祭典`, `蘭法`
  - `新年` is ignored because it is in nested quotation.
  In Telegram, user can still search for it using "新年".

- 【系統】所有支線劇情的HARD冒險將減緩體力消耗＆提升記憶碎片的掉落率！

  - Tags from rules: `支線劇情`, `HARD`
  - Tags from title: `系統`

## Add tag to title

When publishing post to Telegram, we add tags to title.

Currently, there are three possible implementations, and we use the first one.

1. Add all tags before the title, remove square brackets.

    ```
    #轉蛋 #公主祭典 #獎勵轉蛋 #蘭法
    《公主祭典 獎勵轉蛋》★3「蘭法（新年）」期間限定角色登場！舉辦預告！

    #系統 #支線劇情 #HARD
    所有支線劇情的HARD冒險將減緩體力消耗＆提升記憶碎片的掉落率！
    ```

2. Directly tag the title when possible, remove square brackets.

    ```
    #轉蛋
    《#公主祭典 #獎勵轉蛋》★3「#蘭法（新年）」期間限定角色登場！舉辦預告！

    #系統
    所有 #支線劇情 的 #HARD 冒險將減緩體力消耗＆提升記憶碎片的掉落率！
    ```

3. Directly tag the title when possible, keep square brackets.

    ```
    【#轉蛋】《#公主祭典 #獎勵轉蛋》★3「#蘭法（新年）」期間限定角色登場！舉辦預告！
    【#系統】所有 #支線劇情 的 #HARD 冒險將減緩體力消耗＆提升記憶碎片的掉落率！
    ```

## Future Work

In the future, we may consider using machine learning to automatically generate tags for content.