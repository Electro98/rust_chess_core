# Silly Chess Engine (自分PRになるもの)
## 日本語（翻訳）
### 簡単な説明
このプロジェクトには約1年間取り組みました。当初は卒業制作として作成されましたが、当時の完成度に満足できなかったため開発を継続しました。現在ではチェスの全ロジックを正確に実装した完成済みのチェスエンジンであり、数え切れないほどのデバッグ作業を経ているためその正確性に強い自信を持っています。

「フォグ・オブ・ウォー」のロジックを実装していますが、これはマルチプレイヤーゲーム用のコンセプトとして残されているもので、現時点では未使用です。

### アピールポイント
このエンジンはRustでAIツールを一切使わずにゼロから実装しました（学習目的においては、AIツールは手早く雑な作業には適していないと考えているため）。
現在のアーキテクチャは最善とは言えませんが、プロジェクトの現段階では適しており、開発プロセスを通じて進化してきたものです。
エンジンは「メールボックス」型の盤面表現を採用しているため、ビットボードを使用するエンジンと比較すると速度面で劣りますが、それでも十分な高速性を備えています！

### コード構造
ゲームの主要ロジックは`src/core/engine.rs`に実装されています。主要な構造体の定義とチェス固有の全ロジックが含まれています。

シンプルなクライアントのロジックは`src/main.rs`に記述されています。

ネットワーク関連のコードは`src/online_game/`ディレクトリ、`src/server_bin.rs`、`src/client.rs`に配置されています。

ソースコードのコンパイル方法は`ReadMe.md`に記載されていますが、コンパイル済みバイナリは`bin/`ディレクトリに同梱されるべきです。

### 使用方法
ローカルクライアントとサーバーはパラメータを受け取りません。

オンラインクライアントはCLI引数でゲームのURLを受け取ります。
現バージョンではクライアントがルームIDを表示しないため、`RUST_LOG`環境変数に`client=debug`を設定する必要があります。

例:
```shell
RUST_LOG=client=debug ./bin/client "ws://0.0.0.0:3030/ws/{game_id}"
```  
正直に申し上げると、**Windows**環境でのネットワーク機能については完全な動作を保証できませんが、起動自体は可能でありテストもパスしています。

## English (Original)
### Small description
This is my project that I worked on for about one year. Firstly it was created to serve as my diploma project, but I continued it development because I wasn't satisfied with its state in that moment. Currently it is fully ready for use chess engine that implements all logic of chess without errors and I very confident about it because of countless hours of debugging it.
It has logic for "fog of war" that supposed to be in multiplayer game, but for now it's left out.

### Appeal points
This full engine was written from scratch in Rust without use of AI tools (because I think they are great for quick and dirty work but not for learning by yourself).
Current architecture isn't the best but it's good for current project and was evolving throughout the development project.
Engine is using "mailbox" type of field and because of it isn't very comparable in speed with engines using bit-boards but still quite fast!

### Code structure

Main logic of game is located in `src/core/engine.rs`. There are all of definitions for main structures and all of the logic for chess themselves.

Logic of simple client itself can be found in `src/main.rs`.

Network related code are in `src/online_game/`, `src/server_bin.rs` and `src/client.rs`.

Instructions for compiling source code can be found in `ReadMe.md` but compiled binaries should be in `bin/`.

### Usage

Local client and server doesn't accept any parameters.

Online client accept url of current game as cli argument. 
Also currently client doesn't show room id so there is necessity for using `RUST_LOG` environment with value `client=debug`.

Example:
```shell
RUST_LOG=client=debug ./bin/client "ws://0.0.0.0:3030/ws/{game_id}"
```

To be honest, I cannot guarantee that network part on Windows will work flawlessly but It will start and test are passing.
