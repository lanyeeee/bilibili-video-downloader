<p align="center">
    <img src="https://github.com/user-attachments/assets/a66896c7-33a6-463e-81fe-bacca3223191" style="align-self: center"/>
</p>

# 📺哔哩哔哩视频下载器

哔哩哔哩 bilibili B站 视频 下载器，图形界面 + nfo刮削 + 广告标记 + 字幕下载 + 弹幕下载，轻松将视频加入emby等媒体库


## 📥 快速下载

[Release页面](https://github.com/lanyeeee/bilibili-video-downloader/releases)提供了预编译的安装包，直接下载即可使用

**如果本项目对你有帮助，欢迎点个 Star ⭐ 支持！你的支持是我持续更新维护的动力 🙏**

## 🖥️图形界面

![](https://github.com/user-attachments/assets/73d4a9d7-644b-43f2-9b25-66212e8fd9a8)

## ✨ 主要特性

| 特性        | 说明                                                         |
| :---------- | :----------------------------------------------------------- |
| 🖥️图形界面   | 基于 [Tauri v2](https://www.google.com/url?sa=E&q=https%3A%2F%2Fv2.tauri.app%2Fstart%2F)，跨平台、现代、轻量、简洁、易用 |
| ⚡分片下载   | 最大化下载速度，轻松榨干带宽(如果你想的话)，当然也支持限速   |
| 📁自定义命名 | `bv号` `标题` `分P`  `发布时间` `UP昵称`等...自由组合成你喜欢的目录结构与文件命名规则 |
| 🔍视频搜索   | `视频(av/bv)` `番剧(ep/ss)` `课程(ep/ss)` `UP投稿/个人空间(uid)` `收藏夹(fid)` |
| 👤账号相关   | **登录** `Cookie登录` `二维码登录`<br />**内容** `收藏夹` `历史记录` `稍后再看` `追番追剧` |
| 🎬视频下载   | **类型** `视频(合集、分P、充电视频)` `番剧(正片、PV、相关视频)` `课程` <br />**编码** `AVC` `HEVC` `AV1`<br />**画质** `8K` `杜比视界` `HDR` `4K` `AI智能修复`等... |
| 🎵音频下载   | `无损` `杜比全景声` `192K` `132K` `64K`                      |
| 📝字幕下载   | 获取视频所有的CC字幕，以`srt`格式保存                        |
| 🖼️封面下载   | 最高清、无压缩的原始封面图                                   |
| 💬弹幕下载   | 不仅能下载 `xml` `json`格式的原始弹幕，还支持将其转为样式可定制的`ass字幕` |
| 📺NFO刮削    | 还会顺便下载poster和fanart，轻松将视频加入emby等媒体库       |
| 🎞️章节标记   | 将原视频的章节信息嵌入视频文件，使视频在各类播放器中支持章节导航 |
| 🚫广告标记   | 将广告片段以章节的形式嵌入视频文件，配合兼容的播放器可自动跳过广告 |
| ⚙️任务管理   | `断点续传` `批量操作` `继续` `暂停` `重来`  `删除`           |

## 📖 使用方法

这个视频是主要功能的演示

https://github.com/user-attachments/assets/adf84b93-684f-43f3-9948-6ba527213812



## ⚠️关于被杀毒软件误判为病毒

对于个人开发的项目来说，这个问题几乎是无解的(~~需要购买数字证书给软件签名，甚至给杀毒软件交保护费~~)  
我能想到的解决办法只有：

1. 根据下面的**如何构建(build)**，自行编译
2. 希望你相信我的承诺，我承诺你在[Release页面](https://github.com/lanyeeee/bilibili-video-downloader/releases)下载到的所有东西都是安全的

## 🛠️如何构建(build)

构建非常简单，一共就3条命令  
~~前提是你已经安装了Rust、Node、pnpm~~

#### 📋前提

- [Rust](https://www.rust-lang.org/tools/install)
- [Node](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

#### 📝步骤

#### 1. 克隆本仓库

```
git clone https://github.com/lanyeeee/bilibili-video-downloader.git
```

#### 2.安装依赖

```
cd bilibili-video-downloader
pnpm install
```

#### 3.构建(build)

```
pnpm tauri build
```

## 🤝提交PR

**PR请提交至`develop`分支**

**如果想新加一个功能，请先开个`issue`或`discussion`讨论一下，避免无效工作**

其他情况的PR欢迎直接提交，比如：

1. 🔧 对原有功能的改进
2. 🐛 修复BUG
3. ⚡ 使用更轻量的库实现原有功能
4. 📝 修订文档
5. ⬆️ 升级、更新依赖的PR也会被接受

## ⚠️免责声明

- 本工具仅作学习、研究、交流使用，使用本工具的用户应自行承担风险
- 作者不对使用本工具导致的任何损失、法律纠纷或其他后果负责
- 作者不对用户使用本工具的行为负责，包括但不限于用户违反法律或任何第三方权益的行为

## 🙏感谢

[bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect)

[ffmpeg](https://github.com/FFmpeg/FFmpeg)

[danmu2ass](https://github.com/gwy15/danmu2ass)

[BilibiliSponsorBlock](https://github.com/hanydd/BilibiliSponsorBlock)

## 💬其他

任何使用中遇到的问题、任何希望添加的功能，都欢迎提交`issue`或开`discussion`交流，我会尽力解决

