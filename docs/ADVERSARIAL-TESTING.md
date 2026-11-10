# Pruebas adversariales

El release corre `cargo test --locked`. Esa pasada incluye [`tests/adversarial.rs`](../tests/adversarial.rs), que es la suite del protocolo, y los checks del workflow (`actionlint`, `zizmor`, el formato del tag, la versión y las notas). Una pasada escrita hace falta antes de un release que cambie el protocolo HID, la importación JSON o los workflows de GitHub Actions. Un parche pequeño no la necesita. El último informe es [adversarial-reports/v1.0.0.md](adversarial-reports/v1.0.0.md).

Los tests afirman el comportamiento que el programa debe tener. Una aserción que falla es un hallazgo. No se debilita el test para que coincida con un bug, y no se le pone `#[ignore]` hasta triar el hallazgo.

## Reglas

- Los tests no abren el teclado ni la ventana. La escritura real se comprueba al pulsar Escribir, con el teclado conectado.
- `cargo fmt` y `cargo clippy --all-targets -- -D warnings` se quedan limpios.
- Un área es dueña de sus archivos. Dos pasadas no editan el mismo archivo.

## Áreas

| Área | Dueña | Qué ataca |
| ---- | ----- | --------- |
| Protocolo | `tests/adversarial.rs` | Tramas de cable y de dongle, offsets, timeouts que no se tratan como payload, PID y usage page, topes de cada eje, Rapid Trigger, capa Fn, SOCD, DKS, macros, luz, JSON corto o de otro producto, y los comandos que nunca deben enviarse: reset de fábrica, entrada a flash y la calibración que borra la calibración guardada. |
| Actions | Informe, con `actionlint` y `zizmor` en CI y en el release | `${{ }}` dentro de `run:`. Actions clavadas a un tag móvil. Quién puede publicar. Que solo un tag `v*` publica, y que el build es solo Linux. |

`actionlint` 1.7.12 y `zizmor` 1.30.1 (`--persona auditor`) corren en [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) y en [`.github/workflows/release.yml`](../.github/workflows/release.yml). El nombre de archivo del paquete llega por la variable `LABEL`, no interpolado en el texto del script.

## Después de la pasada

1. Reproduce cada hallazgo y tira los duplicados.
2. Crítico y alto: arregla el código, un commit por arreglo, hasta que el test pase.
3. Bajo: deja el comportamiento, anótalo en el README si le importa a quien usa el programa, y dilo en el informe. Un test que solo documenta un hallazgo bajo puede quedarse como test de caracterización.
4. Escribe `docs/adversarial-reports/vX.Y.Z.md` con, para cada hallazgo: id, severidad, estado (arreglado o aceptado), el nombre del test, cómo reproducirlo, y el arreglo o la razón por la que se quedó.
