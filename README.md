# FotoTPM

Scegli le foto e riordinale. Il programma le croppa e le carica automaticamente.

## Installazione

- Installare [il compilatore per il linguaggio Rust](https://www.rust-lang.org/tools/install).
- Scaricare il [codice sorgente](https://github.com/MichaelObvious/foto_tpm/archive/refs/heads/master.zip).
- Estrarre la cartella scaricata.
- Nella cartella estratta, aprire il terminale e digitare il comando seguente per far partire il programma. (La prima volta si dovrà attendere un pochino)

```sh
cargo run --release
```

## Utilizzo

Nella scheda `Dati` inserire:

- Il titolo dell'attività
- La branca (`CASTO`/`LUPI`/`ESPLO`/`PIO`/`SEZIONE`/...)
- La data in cui si è svolta l'attività (giorno, mese e _ultime due cifre_ anno)
- Il server su cui caricarle
- Il nome utente per accedere al server
- La password per accedere al server

Nella scheda `Foto` rilasciare le foto. Riordinarle con le [scorciatoie da tastiera](#scheda-foto).

Una volta terminato il riodinto e la correzione, controllare la correttezza dei `Dati`. Premere poi il tasto `Upload`.

Le foto verranno salvate in una cartella, e poi si potrà scegliere se caricarle o meno sul server.

## Scorciatoie da tastiera

### Scheda `Foto`

| Scorciatoia                     | Effetto                        |
| ------------------------------- | ------------------------------ |
| <kbd>DELETE</kbd>               | rimuovi foto                   |
| <kbd>↑</kbd>                    | foto precedente                |
| <kbd>↓</kbd>                    | foto successiva                |
| <kbd>SHIFT</kbd>+<kbd>↑</kbd>   | anteponi foto                  |
| <kbd>SHIFT</kbd>+<kbd>↓</kbd>   | posponi foto                   |
| <kbd>R</kbd>                    | ruota foto in senso orario     |
| <kbd>SHIFT</kbd>+<kbd>R</kbd>   | ruota foto in senso antiorario |