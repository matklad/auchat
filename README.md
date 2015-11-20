auchat server
=============

Для сборки потребуется [Rust](https://www.rust-lang.org/) и Linux (поддержку
Windows обещают в следующем релизе).

* Сервер `cargo run --release --bin server -- --workers=4`
* Клиент `cargo run --release --bin client -- --login=Alice`
* Бенчмарк  `cargo run --release --bin bench -- --rps --med --large --huge --c10`
