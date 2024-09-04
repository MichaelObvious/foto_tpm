# FotoTPM

Scegli le foto e riordinale. Il programma le ridimensiona e le carica automaticamente.

## Indice

- ["Installazione"](#installazione)
    - [Windows](#windows)
    - [MacOS](#macos)
    - [GNU/Linux](#gnulinux)
- [Utilizzo](#utilizzo)
- [Scorciatoie da tastiera](#scorciatoie-da-tastiera)
    - [Scheda `Foto`](#scheda-foto)

## "Installazione"

### Windows

1. Scaricare il pacchetto corrispondente dalla pagina [`Releases`](https://github.com/MichaelObvious/foto_tpm/releases).
2. Estrarre la cartella scaricata.
3. Fare doppio click sull'applicazione nella cartella estratta per far partire il programma.

### MacOS

**Fino ad ora il programma non è stato testato su MacOS.**

1. Scaricare il pacchetto corrispondente dalla pagina [`Releases`](https://github.com/MichaelObvious/foto_tpm/releases).
    - Se il vostro processore (lo trovate cliccando sull'icona Apple, poi su `Informazioni su questo Mac` e guardando la voce `Processore`) è elencato come:
        - `Apple M`... o `Apple Silicon` o simili, allora scaricare il pacchetto con `aarch64` nel nome;
        - `Apple Intel`... o simili, allora scaricare il pacchetto con `x86_64` nel nome;
2. Estrarre la cartella scaricata.
3. Fare doppio click sull'applicazione nella cartella estratta per far partire il programma.

### GNU/Linux

_Nota: per GNU/Linux funziona anche far andare l'eseguibile per Windows tramite [Wine](https://en.wikipedia.org/wiki/Wine_(software))_.

**Prerequisiti:** gli stessi di [raylib](https://github.com/raysan5/raylib). Fare riferimento a questa [guida](https://github.com/CapsCollective/raylib-cpp-starter/blob/main/docs/InstallingDependencies.md).

1. Installare [il compilatore per il linguaggio Rust](https://www.rust-lang.org/tools/install).
2. Scaricare il codice sorgente.
3. Nella cartella in cui è contenuto il file `Cargo.toml` aprire il terminale e digitare il comando seguente per far partire il programma _(la prima volta si dovrà attendere un pochino)_.

```sh
cargo run --release
```

## Utilizzo

1. Nella scheda `Dati` inserire:
    - Il titolo dell'attività;
    - La branca (`CASTO`/`LUPI`/`ESPLO`/`PIO`/`SEZIONE`/...);
    - La data in cui si è svolta l'attività (giorno, mese e _ultime due cifre_ dell'anno civile);
    - Il server su cui caricarle;
    - Il nome utente per accedere al server;
    - La password per accedere al server;
    - La volontà di caricare le fotografie in risoluzione maggiore (o _"HD"_, 1200x1600 px) oppure no (600x800 px).
2. Nella scheda `Foto` rilasciare le foto. I formati supportati attualmente sono `JPEG` e `PNG`. _(Mentre vengono caricate, le foto vengono già ridimensionate e ritagliate automaticamente per essere della dimensione desiderata)_
3. Riordinare le foto con le [scorciatoie da tastiera](#scheda-foto).
4. Una volta terminato il riordino e la correzione, controllare la correttezza dei `Dati`.
5. Nella scheda `Foto` premere il tasto `Upload`.
6. Le foto verranno salvate in una cartella, e poi si potrà scegliere se caricarle o meno sul server.
7. Ogni volta che viene chiusa l'applicazione, verrà salvato (nella _working directory_ del programma) un file `fototpm-imglist_`...`.txt` che contiene una lista di tutte le immagini selezionate. Questo file può essere riutilizzato per riprendere il lavoro in un secondo momento, rilasciando il file nell'applicazione aperta.

## Scorciatoie da tastiera

### Scheda `Foto`

| Scorciatoia                     | Effetto                        |
| ------------------------------- | ------------------------------ |
| <kbd>DELETE</kbd>               | rimuovi foto                   |
| <kbd>↑</kbd>                    | foto precedente                |
| <kbd>↓</kbd>                    | foto successiva                |
| <kbd>CTRL</kbd>+<kbd>↑</kbd>    | salto in avanti                |
| <kbd>CTRL</kbd>+<kbd>↓</kbd>    | salto indietro                 |
| <kbd>SHIFT</kbd>+<kbd>↑</kbd>   | anteponi foto                  |
| <kbd>SHIFT</kbd>+<kbd>↓</kbd>   | posponi foto                   |
| <kbd>R</kbd>                    | ruota foto in senso orario     |
| <kbd>SHIFT</kbd>+<kbd>R</kbd>   | ruota foto in senso antiorario |