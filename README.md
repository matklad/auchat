auchat server
=============

Для сборки потребуется [Rust](https://www.rust-lang.org/) и Linux ( ).

* Сервер `cargo run --release --bin server -- --workers=4`
* Клиент `cargo run --release --bin client -- --login=Alice`
* Бенчмарк  `cargo run --release --bin bench -- --seq --c10k`



