---
nombre: estado-local-en-progress
descripcion: Estado del arnes en progress/: dotfile, mtime de reloj, vacio = default.
triggers: [progress, .last_nudge, .last_autocheck, debounce, contador, stamp, backoff, mtime]
relacionadas: [docs-generados-por-el-instalador]
origen: [18]
usos: 0
ultimo_uso:
ultima_actualizacion: 2026-08-16
estado: activa
---

## Cuando aplica

Cuando una feature necesita que el arnes **recuerde algo entre invocaciones** que
no es parte del backlog ni de la documentacion: un contador, un debounce, un
nivel de escalada, "cuando fue la ultima vez que...".

Tambien aplica al reves: cuando estas por agregar un campo a `feature_list.json`
para algo que en realidad es estado efimero de la maquina. Eso no va ahi.

## Procedimiento

1. Un **dotfile por concepto**, bajo `progress/`, declarado como campo en
   `HarnessPaths` (`paths.rs`) junto a `autocheck_stamp` y `nudge_stamp`. Toda la
   resolucion de rutas vive en un solo lugar.
2. **El `mtime` es el reloj.** No guardes timestamps adentro del archivo: para
   "¿cuanto pasó desde la ultima vez?" alcanza `mtime_f64(&path)`, y escribir el
   archivo actualiza el reloj gratis.
3. **El contenido es el estado chico**: un entero, un `<clave>:<valor>`. Texto
   plano de una linea, parseado con `unwrap_or(default)`.
4. **Toda lectura degrada al default.** Archivo ausente, vacio, con basura o con
   permisos rotos: valor por defecto y seguir. Este estado nunca puede hacer
   fallar un comando.
5. **Ambito**: si el estado depende de la feature activa, guarda el id adentro
   (`<id>:<n>`) y reinicia cuando no coincide. Es mas barato y mas robusto que
   limpiar el archivo desde `start` y `close`.

## Pitfalls

- **Cambiar el formato de un dotfile que ya existe en instalaciones vivas.**
  `progress/.last_nudge` existia como archivo VACIO (solo importaba su mtime).
  Al empezar a guardarle el nivel de backoff, un `parse::<u32>()` sin
  `unwrap_or(0)` habria roto —o peor, silenciado— el aviso en toda instalacion
  que ya lo tenia. Un archivo vacio del formato viejo **tiene que** leerse como el
  default del formato nuevo, y eso se testea explicitamente.
- **Escribir en cada invocacion.** Este estado lo toca el hook `PostToolUse`, o
  sea una vez por tool-use. Antes de escribir, chequea si hace falta: reescribir
  un valor que ya esta bien tambien actualiza el mtime, y si el mtime es tu reloj,
  acabas de correr el reloj sin querer.
- **Crear el archivo cuando la feature esta apagada.** Si el comportamiento
  depende de una carpeta o de una regla (por ejemplo `docs/lecciones/`), poné la
  guarda ANTES de tocar el filesystem: un proyecto que no usa la feature no
  deberia ganar archivos nuevos en `progress/`.
- **Confundirlo con memoria.** `progress/history.md` es bitacora (append-only, se
  lee), los dotfiles son estado (se pisan, nadie los lee a mano). Si alguien va a
  querer leerlo, no es un dotfile.

## Verificacion

```bash
# El formato viejo se sigue leyendo (compatibilidad):
: > progress/.last_nudge && sh harness_cli nudge && echo "exit=$?"   # -> 0

# El estado no se crea cuando la feature esta apagada:
ls progress/.nudge_lecciones 2>/dev/null || echo "no existe: correcto"

# Y nada de esto puede romper un comando:
printf 'basura' > progress/.last_nudge && sh harness_cli nudge; echo "exit=$?"  # -> 0
```

En Rust, los tests usan `filetime::set_file_mtime` para correr el reloj sin
esperar: es dev-dependency del repo y evita tests lentos o flaky.
