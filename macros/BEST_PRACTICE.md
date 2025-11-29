# マクロ開発のベストプラクティス


[[Rust] Procedural Macros を実装するときに準備していること](https://zenn.dev/linnefromice/articles/prepare-to-implement-rust-procedural-macros)
より


## エントリポイントとマクロロジックを分離する

- エントリポイントとなる`#[proc_macro]`属性を付与した関数
- 実際に入力`TokenStream`と出力の`TokenStream`を生成するロジックの関数

これらを分離し、、エントリポイントとなる関数はロジックの関数をコールするだけにする

### これのメリット

- 外部に公開するマクロ関数が何であるかをわかりやすくする
- テストを行う際に効果的

## `syn`,`proc_macro2`ベースにする

ロジック側で`syn`,`proc_macro2`クレートをベースにロジック部分を実装していくのが良い。

`syn`,`quote`,`proc_macro2`などマクロに関する便利クレートがあり、基本的にこれを駆使してマクロコードを生成。