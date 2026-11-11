# hekit

Configurador de escritorio para el teclado **AULA MINI 60 HE PRO** en Linux. Habla el protocolo HID del driver oficial (cable `0C45:80A2` y receptor `0C45:FEFE`) y guarda o carga la configuración en JSON.

La ventana está en español. Sirve para la capa Fn, la actuación, el Rapid Trigger y la elección de eje. Jade Pro usa el id 2 con tope de firmware 3.30 mm (el recorrido real del switch es 3.50 mm). Jade Ultra no tiene id: se guarda como el id 2, el deslizador no pasa de 3.20 mm y hekit recuerda qué teclas son Ultra en `~/.config/hekit/switch-map.json`.

Rapid Trigger está activo cuando la presión y la liberación son distintas de cero. Apagarlo escribe 0 y 0. El byte de flags no es el interruptor de Rapid Trigger: el bit 0 es activación rápida de todo el recorrido, el bit 1 es optimización de fondo y el bit 2 es Rampage. Esos dos últimos se conservan y no se muestran, igual que en el driver web. Un recorrido 0 leído se muestra como 1.20 mm y se vuelve a escribir como 0 hasta que lo muevas. Un byte de eje 0 es el switch de fábrica (`AxialModel`, bytes 20-21 de la información del dispositivo).

## Qué configura

- Actuación por tecla y Rapid Trigger, con presión y liberación separadas
- Los ocho ejes del driver actual, más el perfil Jade Ultra
- Capa normal y capa que se usa al mantener Fn
- SOCD, rapid snap, DKS, mod-tap, toggle y macros
- Luz guardada en el teclado y color por tecla
- Tasa de informe, suspensión, zonas muertas, estabilidad y calibración adaptativa

No envía reset de fábrica, entrada a flash ni la calibración que borra la calibración guardada. No anima luces en tiempo real.

## Instalar

El uso normal es el paquete. Genera el `.deb` y lo instala:

```sh
cargo install cargo-deb --locked
cargo deb
sudo apt install ./target/debian/hekit_*.deb
```

Eso deja el comando `hekit` y una entrada en el menú de aplicaciones. Durante la instalación el paquete copia la regla udev a `/usr/lib/udev/rules.d/70-hekit.rules` y, con udev en marcha, ejecuta `udevadm control --reload-rules` y `udevadm trigger --subsystem-match=hidraw`. No hay que copiar la regla a mano. Al desinstalar, dpkg quita la regla y vuelve a recargar udev.

Si el teclado ya estaba enchufado, desconéctalo y vuelve a conectarlo una vez para que el permiso aplique al dispositivo que ya estaba abierto.

Comprueba la instalación sin abrir la ventana:

```sh
hekit --check
hekit --dump
```

`--dump` solo lee. Imprime firmware, `AxialModel`, actuación, flags y la capa Fn, y guarda un respaldo en `~/.local/share/hekit/backups/` (o en `HEKIT_BACKUP_DIR`). Cada lectura y cada escritura también dejan un respaldo ahí. Escribir manda únicamente las tablas que cambiaron desde la última lectura, después de un resumen, y las relee. Restaurar un respaldo reescribe solo las tablas distintas.

Si el cable y el receptor están conectados a la vez, hekit usa el cable. El PID `80B2` se reconoce y no se escribe: no hay una captura de ese hardware.

## Releases

El paquete publicado sale de un tag `vMAJOR.MINOR.PATCH` en GitHub. El primero es `v1.0.0`. Cómo escribir las notas y empujar el tag está en [docs/HOW-TO-CHANGELOG.md](docs/HOW-TO-CHANGELOG.md).

Cada release de Linux x86-64 trae un `.deb`, un `.rpm`, un AppImage y un `.tar.gz`, más `checksums.txt`. Necesitan glibc 2.39 o más nuevo (Ubuntu 24.04, Debian 13, Fedora 40 o posterior). El `.deb` y el `.rpm` instalan la regla udev y la recargan. El AppImage y el tar traen la regla para copiarla a mano. Las notas de `v1.0.0` están en [docs/release-notes/v1.0.0.md](docs/release-notes/v1.0.0.md).

## Desarrollar

Rust 1.98.1, fijado en `rust-toolchain.toml`, y estas librerías de desarrollo (las mismas que instala CI):

```sh
sudo apt install pkg-config libudev-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libssl-dev
```

`hidapi` usa el backend `linux-native`, que enlaza libudev. Sin `libudev-dev`, `libudev-sys` aborta en `pkg-config --libs --cflags libudev` porque no encuentra `libudev.pc`. No hace falta definir `PKG_CONFIG_PATH`: el paquete deja el archivo en la ruta por defecto.

```sh
cargo run
cargo run -- --check
```

## JSON

Exportar escribe un archivo `mini60-config` versión 2: las tablas crudas, sin pérdida, y una vista con nombres, ejes y milímetros. La versión 1 se sigue importando. Importar también acepta un perfil de hec cuando trae `RTlist` (como lista o como objeto `{"0": …}`) y, si viene, `allFnPack`. Importar no escribe solo: carga cambios pendientes sobre la última lectura y muestra el diff. Un archivo corto, una versión distinta de 1 o 2, o un product id que no sea `0x80A2` se rechaza y no se escribe nada al teclado.

Antes de escribir hace falta haber leído el teclado o haber importado un perfil, para no pisar tablas que todavía no se leyeron.

## Pruebas

```sh
cargo test
```

Cubren los topes de cada eje, la trama de cable y de dongle, la capa Fn, los pares SOCD, el JSON y los comandos que nunca deben enviarse. `tests/adversarial.rs` es la pasada adversarial de ese protocolo; el release la corre con `cargo test --locked`. Cuándo repetirla, y qué se mira de los workflows, está en [docs/ADVERSARIAL-TESTING.md](docs/ADVERSARIAL-TESTING.md). No hay un teclado físico en esa pasada: la escritura real se comprueba al pulsar Escribir con el teclado conectado. La aplicación relee cada tabla y avisa si no coincide.
