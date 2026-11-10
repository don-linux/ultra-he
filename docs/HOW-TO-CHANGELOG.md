# Cómo escribir las notas y publicar un release

Un release se publica al empujar un tag de Git, y solo así. Los commits y los merges a `main` no construyen ni publican nada. Cuando un tag `v*` llega a GitHub, [`release.yml`](../.github/workflows/release.yml) lo comprueba, construye los paquetes de Linux y crea el GitHub Release, con estas notas como descripción y los paquetes como descargas.

## Versiones y tags

Las versiones siguen [Semantic Versioning](https://semver.org): `MAJOR.MINOR.PATCH`. El tag es la versión con una `v` delante.

| Tag | Significado |
| --- | --- |
| `v1.0.0` | Primera versión estable de hekit |
| `v1.0.1` | Un arreglo sobre la estable |
| `v1.1.0` | Funcionalidad nueva compatible |
| `v2.0.0` | Un cambio que rompe lo anterior |

- Solo números: `v1.0.0` vale, `v1.0.0-rc.1` o `1.0.0` los rechaza el workflow.
- El tag tiene que coincidir con `version` en [`Cargo.toml`](../Cargo.toml): `v1.0.1` exige `version = "1.0.1"`. Si no, el workflow se detiene.
- Usa tags anotados (`git tag -a`).
- Un tag ya publicado no se mueve ni se reutiliza. Si un release salió mal, publica el siguiente patch.

## Dónde van las notas

Un archivo Markdown por versión, con el nombre del tag:

```text
docs/release-notes/
├── v1.0.0.md
└── v1.1.0/          # opcional: imágenes para v1.1.0.md
    └── teclado.png
```

El workflow publica `docs/release-notes/<tag>.md` tal cual, como descripción del release. Sin ese archivo no hay release. Las notas automáticas de GitHub están apagadas.

Las imágenes viven en una carpeta con el nombre de la versión. Enlázalas a través del tag para que el enlace no se rompa:
`https://raw.githubusercontent.com/don-linux/ultra-he/v1.1.0/docs/release-notes/v1.1.0/teclado.png`

## Qué va en el archivo

Escríbelo para quien decide si actualiza, no para quien revisa el diff. Di qué cambió para esa persona.

Los nombres de sección siguen [Keep a Changelog](https://keepachangelog.com). Usa solo las que apliquen, en este orden:

| Sección | Para |
| --- | --- |
| Added | Funciones nuevas |
| Changed | Cambios de comportamiento |
| Deprecated | Lo que se va a quitar después |
| Removed | Lo que este versión quita |
| Fixed | Arreglos |
| Security | Vulnerabilidades corregidas |

Cada entrada empieza con una frase corta en negrita y sigue con una o dos frases. Acredita y enlaza el pull request o el issue: `Por @alguien; gracias a @quien-lo-reportó. (#12)`.

## Plantilla

Copia esto en `docs/release-notes/vX.Y.Z.md` y sustituye cada `X.Y.Z` y `vA.B.C` (el tag anterior):

````markdown
hekit X.Y.Z <una o dos frases: de qué va este release>.

**Descarga hekit:** [AppImage](https://github.com/don-linux/ultra-he/releases/download/vX.Y.Z/hekit-vX.Y.Z-x86_64.AppImage) · [.deb](https://github.com/don-linux/ultra-he/releases/download/vX.Y.Z/hekit-vX.Y.Z-x86_64.deb) · [.rpm](https://github.com/don-linux/ultra-he/releases/download/vX.Y.Z/hekit-vX.Y.Z-x86_64.rpm) · [.tar.gz](https://github.com/don-linux/ultra-he/releases/download/vX.Y.Z/hekit-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz)

Los paquetes de Linux necesitan glibc 2.39 o más nuevo: Ubuntu 24.04, Debian 13, Fedora 40 o posterior.

- **AppImage**, para probarlo sin instalarlo: `chmod +x hekit-vX.Y.Z-x86_64.AppImage` y ejecútalo. La regla udev va dentro; hay que copiarla a mano a `/usr/lib/udev/rules.d/70-hekit.rules` y recargar udev.
- **Debian y Ubuntu:** `sudo apt install ./hekit-vX.Y.Z-x86_64.deb`. Instala la regla udev y la recarga.
- **Fedora:** `sudo dnf install ./hekit-vX.Y.Z-x86_64.rpm` (openSUSE Tumbleweed: `sudo zypper install` con el mismo archivo). Instala la regla udev y la recarga.
- **tar.gz:** el binario, el `.desktop`, la licencia y la regla udev. La regla hay que copiarla a mano.

Si el teclado ya estaba enchufado, desconéctalo y vuelve a conectarlo una vez para que el permiso aplique al dispositivo que ya estaba abierto.

![Qué muestra la captura](https://raw.githubusercontent.com/don-linux/ultra-he/vX.Y.Z/docs/release-notes/vX.Y.Z/teclado.png)

## Added

- **Frase corta.** Detalle. Por @alguien. (#12)

## Fixed

- **Frase corta.** Detalle. Por @alguien; gracias a @quien-lo-reportó. (#13)

**Changelog completo**: https://github.com/don-linux/ultra-he/compare/vA.B.C...vX.Y.Z
````

Quita la línea de la captura cuando no hay imagen. El primer release no lleva la línea de changelog completo porque no hay tag anterior.

## Lista para publicar

1. **Abre un pull request a `main`** con:
   - `version = "X.Y.Z"` en `Cargo.toml` (corre `cargo build` para que `Cargo.lock` lo siga);
   - `docs/release-notes/vX.Y.Z.md`.
2. **Espera el CI.** [`ci.yml`](../.github/workflows/ci.yml) corre el formato, Clippy y los tests, y construye los paquetes de Linux con los mismos pasos que un release. Los paquetes se pueden bajar de los **Artifacts** del run durante 7 días; pruébalos antes de etiquetar.
3. **Mezcla** el pull request.
4. **Etiqueta el commit mezclado y empuja el tag** al remoto de GitHub:

   ```bash
   git switch main
   git pull origin main
   git tag -a vX.Y.Z -m "hekit X.Y.Z"
   git push origin vX.Y.Z
   ```

5. **Mira el workflow Release** en la pestaña Actions. Cuando termina, el release está en la [página de releases](https://github.com/don-linux/ultra-he/releases).

Antes de construir, el workflow comprueba que:

- el tag tiene el formato correcto;
- el tag coincide con `Cargo.toml`;
- el archivo de notas existe;
- el commit etiquetado está en `main`;
- el formato, Clippy y los tests pasan.

## Si el workflow Release falla

El release se crea solo después de que Linux haya construido, así que un fallo deja un tag sin release. Arréglalo y vuelve a etiquetar la misma versión:

```bash
git push origin :refs/tags/vX.Y.Z   # borra el tag en GitHub
git tag -d vX.Y.Z                   # bórralo en local
```

Corrige el problema en un pull request nuevo, mézclalo y repite el paso 4.

Hazlo solo mientras no exista un release para ese tag. Cuando el release ya se publicó, alguien pudo descargarlo: déjalo y publica el siguiente patch.

## Qué trae cada release

| Archivo | Qué es |
| --- | --- |
| `hekit-vX.Y.Z-x86_64.AppImage` | Linux x86-64, corre sin instalarse. La regla udev va dentro y hay que copiarla a mano |
| `hekit-vX.Y.Z-x86_64.deb` | Debian, Ubuntu y derivados. Instala la regla udev y la recarga |
| `hekit-vX.Y.Z-x86_64.rpm` | Fedora y openSUSE Tumbleweed. Instala la regla udev y la recarga |
| `hekit-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Linux x86-64: binario, `.desktop` y regla udev para copiar a mano |
| `checksums.txt` | SHA-256 de cada archivo de arriba |

Todo se construye en Ubuntu 24.04 y necesita glibc 2.39 o más nuevo (Ubuntu 24.04, Debian 13, Fedora 40 o posterior). Cuando 24.04 deje de tener soporte, mueve el runner en [`build.yml`](../.github/workflows/build.yml) al siguiente LTS y actualiza esta línea de glibc.
