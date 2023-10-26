# FotoTPM

Scegli le foto e riordinale. Il programma le croppa e le carica automaticamente.

## "Installazione"

### Windows

1. Scaricare il pacchetto corrispondente dalla pagina [`Releases`](https://github.com/MichaelObvious/foto_tpm/releases)
2. Estrarre la cartella scaricata.
3. Fare doppio click sull'applicazione nella cartella estratta per far partire il programma.

### MacOS

**Prerequisiti:** [Wine](https://it.wikipedia.org/wiki/Wine). Per l'installazione fare capo a questa [guida](https://wiki.winehq.org/MacOS), e in particolare alla sezione `Installing Wine packages using homebrew`.

1. Scaricare il pacchetto che termina in **`_win64.zip`** dalla pagina [`Releases`](https://github.com/MichaelObvious/foto_tpm/releases)
2. Estrarre la cartella scaricata.
3. Aprire il terminale nella cartella estratta.
4. Per far partire il programma digitare il seguente comando nel terminale:

```
wine foto_tre_pini.exe
```

### Linux

**Prerequisiti:** gli stessi di [raylib](https://github.com/raysan5/raylib). Fare riferimento a questa [guida](https://github.com/CapsCollective/raylib-cpp-starter/blob/main/docs/InstallingDependencies.md).

1. Installare [il compilatore per il linguaggio Rust](https://www.rust-lang.org/tools/install).
2. Scaricare il [codice sorgente](https://github.com/MichaelObvious/foto_tpm/archive/refs/heads/master.zip).
3. Estrarre la cartella scaricata.
4. Nella cartella estratta, aprire il terminale e digitare il comando seguente per far partire il programma. (La prima volta si dovrà attendere un pochino)

```sh
cargo run --release
```

## Utilizzo

1. Nella scheda `Dati` inserire:
    - Il titolo dell'attività
    - La branca (`CASTO`/`LUPI`/`ESPLO`/`PIO`/`SEZIONE`/...)
    - La data in cui si è svolta l'attività (giorno, mese e _ultime due cifre_ anno)
    - Il server su cui caricarle
    - Il nome utente per accedere al server
    - La password per accedere al server
2. Nella scheda `Foto` rilasciare le foto. _(Mentre vengono caricate, le foto vengono già ridimensionate e ritagliate automaticamente per essere 600x800 o 800x600)_
3. Riordinare le foto con le [scorciatoie da tastiera](#scheda-foto).
4. Una volta terminato il riodinto e la correzione, controllare la correttezza dei `Dati`.
5. Nella scheda `Foto` premere il tasto `Upload`.
6. Le foto verranno salvate in una cartella, e poi si potrà scegliere se caricarle o meno sul server.

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