# suno-cli 中文使用指南

> 命令行版 Suno AI 音乐生成工具，支持 v5.5。本文档覆盖安装、认证、常用命令和踩坑指南。

---

## 目录

1. [安装](#1-安装)
2. [认证](#2-认证)
3. [常用命令速查](#3-常用命令速查)
4. [上传本地音频（upload）](#4-上传本地音频upload)
5. [翻唱（cover）](#5-翻唱cover)
6. [从零生成（generate / describe）](#6-从零生成generate--describe)
7. [其他命令](#7-其他命令)
8. [常见问题与踩坑](#8-常见问题与踩坑)

---

## 1. 安装

### 前置条件

- **Rust**（rustc >= 1.85）—— 如果没有：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Chromium 浏览器**（captcha 解题需要）—— `sudo snap install chromium`
- **Suno 账号**（Pro Plan 推荐 credits 充足）

### 编译安装

```bash
cd /home/liugz/suno-cli-src
cargo install --path . --force
```

安装完成后二进制在 `~/.cargo/bin/suno`。

验证安装：
```bash
suno --version
```

---

## 2. 认证

suno-cli 没有官方 API key，靠浏览器 session 认证。**只需认证一次**，之后自动刷新。

### 方式 A：浏览器自动提取（推荐）

如果你在本机浏览器上已经登录了 suno.com：

```bash
suno auth --login
```

会自动从 Chrome / Chromium / Firefox / Edge 提取 session 凭证。

### 方式 B：手动粘贴 JWT

如果自动提取失败（比如服务器环境）：

1. 浏览器打开 suno.com 并登录
2. F12 打开开发者工具 → Network 标签
3. 刷新页面，搜索 `studio-api-prod`
4. 找到任意请求，复制 `Authorization: Bearer xxx` 中 Bearer 后面的 JWT
5. 运行：
```bash
suno auth --jwt "粘贴JWT"
```

### 验证认证成功

```bash
suno credits
# 输出类似：Pro Plan, Credits: 2980
```

---

## 3. 常用命令速查

| 命令 | 用途 | 示例 |
|------|------|------|
| `suno upload` | 上传本地音频 | `suno upload 歌曲.mp3` |
| `suno cover` | 翻唱（用已有音频） | `suno cover <ID> --tags "pop" --wait` |
| `suno generate` | 自定义歌词生成 | `suno generate --title "歌名" --tags "pop" --lyrics-file 歌词.txt --wait` |
| `suno describe` | 描述模式生成（Suno 写词） | `suno describe --prompt "一首关于夏天的歌" --wait` |
| `suno list` | 查看已有歌曲 | `suno list` |
| `suno download` | 下载歌曲 | `suno download <ID> --output ./songs/` |
| `suno credits` | 查看 credits 余额 | `suno credits` |
| `suno stems` | 提取人声/伴奏 | `suno stems --clip-id <ID>` |

---

## 4. 上传本地音频（upload）

把本地 mp3/wav/flac 等音频文件上传到 Suno，用于后续翻唱（cover）。

### 基本用法

```bash
suno upload /path/to/song.mp3
```

支持的格式：**mp3, wav, flac, ogg, m4a, aac**

### 上传过程说明

上传不是瞬间完成的，CLI 会自动完成以下步骤：

```
1. 获取 S3 上传凭证 → 2. 上传文件到 S3 → 3. 通知 Suno 处理 → 4. 轮询状态
```

正常输出示例（成功）：
```
Uploading mysong.mp3 (3.2 MB, mp3)...
Got upload slot (id: abc123...), sending file to S3...
File uploaded to S3. Notifying Suno...
Waiting for Suno to process the upload...
[5s] Status: processing
[10s] Status: passed_audio_processing
[15s] Status: complete
Status: ✅ complete
```

### ⚠️ 版权检测（重要！）

**Suno 会对上传的音频做版权审核。** 知名歌曲（如刘若英《后来》、周杰伦《晴天》等）大概率会被检测到并拦截。

版权被拦时的输出：
```
Uploading 后来_刘若英_原曲.mp3 (10.7 MB, mp3)...
Got upload slot (id: xxx...), sending file to S3...
File uploaded to S3. Notifying Suno...
Waiting for Suno to process the upload...
[5s] Status: processing
[10s] Status: passed_audio_processing
[20s] Status: error
⚠ Upload processing failed — likely flagged as copyrighted material.
```

**关键点：**
- 上传到 S3 **本身是成功的**，但 Suno 后台审核后判定侵权
- 审核时间约 20-30 秒
- API **不会返回**"版权侵权"之类的明确提示，只是 status 变成 `error` 或 404
- 网页端会弹 toast 提示，CLI 端靠 status 轮询检测
- **结论：知名歌曲基本传不上去**

### 上传成功后怎么用

上传成功会返回一个 `upload_id`，可以直接用于 cover 命令：

```bash
# 上传
suno upload mysong.mp3
# 返回 upload_id: abc123-...

# 直接用 upload_id 做 cover
suno cover abc123-... --tags "pop, acoustic" --title "My Cover" --wait --download ./output/
```

> suno-cli 会自动检测是 upload_id 还是 clip_id，无需手动转换。

---

## 5. 翻唱（cover）

基于已有音频（Suno 生成的 clip 或上传的音频）创建翻唱版本。

### 基本用法

```bash
suno cover <CLIP_ID> --tags "新风格" --title "翻唱标题" --wait
```

**⚠️ 注意：CLIP_ID 是位置参数，直接写 ID，不要加 `--clip-id`！**

```bash
# ✅ 正确
suno cover abc123-... --tags "jazz, piano" --wait

# ❌ 错误
suno cover --clip-id abc123-... --tags "jazz, piano" --wait
```

### 完整参数

```bash
suno cover <CLIP_ID> \
  --tags "pop, acoustic, warm" \        # 风格标签
  --title "我的翻唱版" \                  # 歌曲标题
  --lyrics "[Verse]\n新的歌词..." \       # 自定义歌词（可选）
  --lyrics-file cover_lyrics.txt \       # 或从文件读取歌词
  --vocal female \                       # 声音性别（male/female）
  --weirdness 40 \                       # 实验程度（0-100）
  --model v5.5 \                         # 模型版本
  --wait \                               # 等待生成完成
  --download ./output/                   # 自动下载到目录
```

### 上传 + 翻唱完整流程

```bash
# 1. 上传音频
suno upload /path/to/original.mp3

# 2. 用返回的 upload_id 做 cover
mkdir -p ./cover-output
suno cover <UPLOAD_ID> --tags "acoustic, folk" --title "Acoustic Cover" --wait --download ./cover-output/
```

### ⚠️ Cover 也可能被版权拦

Cover 歌词如果涉及知名歌曲版权，Suno 也会异步检测：
- 提交成功 → clip 进入生成队列
- 后台检测 → 发现歌词侵权 → clip 状态变 `error`
- CLI 会自动检测并报错

---

## 6. 从零生成（generate / describe）

### generate —— 自定义歌词模式

```bash
# 用歌词文件
suno generate \
  --title "周末写代码" \
  --tags "indie rock, guitar, upbeat" \
  --lyrics-file lyrics.txt \
  --vocal male \
  --wait --download ./songs/

# 直接写歌词
suno generate \
  --title "夏日" \
  --tags "pop, electronic" \
  --lyrics "[Verse]\n阳光洒在窗台上\n夏天来了\n[Chorus]\n我们一起去海边" \
  --wait
```

### describe —— 描述模式（Suno 自动写词）

```bash
suno describe \
  --prompt "一首关于深夜写代码的慵懒 lo-fi 歌曲" \
  --title "深夜程序员" \
  --tags "lo-fi, chill, beats" \
  --wait --download ./songs/
```

### lyrics —— 只生成歌词（不消耗 credits）

```bash
suno lyrics --prompt "一首关于夏天的歌"
```

---

## 7. 其他命令

### 查看和搜索

```bash
suno list                # 列出我的歌曲
suno list --json         # JSON 格式（方便脚本处理）
suno search "关键词"     # 按标题/标签搜索
suno info <clip_id>      # 查看单曲详情
suno credits             # 查看 credits 余额
suno models              # 查看可用模型
```

### 下载

```bash
suno download <clip_id> --output ./downloads/    # 下载到指定目录
suno download <id1> <id2> <id3>                  # 批量下载
```

> 下载的 MP3 会自动嵌入歌词（ID3 标签）。

### 续写/延长

```bash
suno extend --clip-id <id> --at 120    # 从第120秒续写
```

### 提取人声/伴奏

```bash
suno stems --clip-id <id>    # 分离人声和伴奏
```

### 管理歌曲

```bash
suno delete <clip_id>                    # 删除
suno publish <clip_id>                   # 设为公开
suno set <clip_id> --title "新标题"      # 修改标题
suno set <clip_id> --lyrics-file 新词.txt  # 修改歌词
```

---

## 8. 常见问题与踩坑

### Q: 上传知名歌曲被拦怎么办？

Suno 的版权检测目前比较严格。已知歌曲（华语/欧美热门）大概率会被拦。
- 可以尝试用不太知名的独立音乐人的作品
- 或者用 Suno 自己生成的 clip 作为源音频做 cover
- upload 的版权检测是**异步的**，CLI 会自动检测并提示

### Q: `--download` 报错 "No such file or directory"

下载目录必须**提前创建**：
```bash
mkdir -p ./output
suno cover <ID> --download ./output/ --wait
```

### Q: cover 命令报错 "unexpected argument '--clip-id'"

cover 的 clip ID 是**位置参数**，直接写 ID：
```bash
suno cover <ID> --tags "..." --wait    # ✅
suno cover --clip-id <ID> --tags "..."  # ❌
```

### Q: 生成一直没反应

captcha 解题 + 生成等待可能 2-5 分钟，期间没有 stdout 输出，**不是卡了**。加上 `--wait` 参数会等待完成。

### Q: JWT 过期

JWT 自动刷新，但如果 Clerk session 也过期了，需要重新认证：
```bash
suno auth --login
```

### Q: hCaptcha 解不了

需要 Chromium 浏览器 + X11 显示环境（不能用 headless）。服务器环境用 Xvfb：
```bash
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99
```

---

## 项目信息

- **Fork 仓库**：https://github.com/iinlanjian/suno-cli
- **上游仓库**：https://github.com/paperfoot/suno-cli
- **本地源码**：`/home/liugz/suno-cli-src/`
- **编译命令**：`cd ~/suno-cli-src && cargo install --path . --force`
