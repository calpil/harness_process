# Veredicto del reviewer - Feature #16: atlassian_auto_push

Veredicto: **approved**
Fecha: 2026-08-16
Spec: `docs/spec-feature-16-atlassian-auto-push.md` (Estado: approved, 29 AC)
Evidencia: `docs/impl-16.md`

## Verificacion oficial

| Comando | Resultado |
| --- | --- |
| `cargo test` | 123 unit + 43 integracion = **166 en verde** |
| `cargo clippy --all-targets -- -D warnings` | limpio |
| `bash tests/setup_smoke.sh` | exit 0, con los bloques nuevos de Atlassian #15 y #16 |
| `./harness_check.sh` | limpio |

## Cobertura de los AC

29 de 29 con evidencia; 12 verificados **en real** contra
`calpil.atlassian.net`. Solo AC-21 y AC-22 (crear proyecto/space) quedan
implementados y no ejecutados: correrlos habria creado estructura
organizacional real en el sitio del usuario sin necesidad. El camino de error
(AC-23) si esta cubierto por el manejo comun de `ApiError`.

Lo central — que el flujo empuje solo — se probo de punta a punta: un `add`
disparo backfill, creo el epic y la historia en Jira y publico las paginas en
Confluence sin que nadie escribiera un comando de Atlassian.

## Constitution

- **Articulo 1**: tests nuevos junto al codigo tocado (unit del interruptor y
  del lock, 7 de integracion, 2 bloques nuevos en el smoke) y los cuatro
  comandos oficiales en verde.
- **Articulo 2**: spec aprobado antes de implementar. El alcance crecio DOS
  veces durante la feature (validacion + creacion, y despues backfill completo);
  las dos veces se actualizo el spec, se mostro el delta y se pidio aprobacion
  antes de seguir. La segunda aprobacion quedo en `progress/history.md` porque
  el sello no se duplica.
- **Articulo 3**: D1..D9 citan sus AC-n; `impl-16.md` y este veredicto se
  organizan por AC.
- **Articulo 4**: el worker hereda el entorno pero no imprime el token en
  `last-push.log` (su salida es la del propio worker, que nunca lo escribe);
  `status` sigue diciendo solo presente/ausente. Exit codes estables: ninguna
  transicion cambia el suyo por culpa del envio.
- **Articulo 5**: quince decisiones registradas (OBS-1..OBS-15), incluidas dos
  que el USUARIO cambio sobre la marcha: OBS-12 reemplaza a OBS-6 (de "solo lo
  nuevo" a "cargar todo") y OBS-14 descarta la propuesta del implementer de
  omitir las subtasks de lo ya cerrado. Ninguna se implemento con la
  observacion abierta.
- **Articulo 6**: sin dependencias nuevas (el worker reusa `ureq` de la #15);
  `templates/` propagado (UPDATING.md y la guia); paridad ps1 escrita.

## Reparos / observaciones del reviewer

1. **Dos defectos propios detectados por los tests** (documentados en
   `impl-16.md`): los tests tomaban las credenciales reales de la maquina — el
   smoke llego a crear issues de verdad en ADR — y los flags de creacion del
   instalador quedaban reseteados por un bloque duplicado. Ambos corregidos, con
   el aislamiento de `HOME` como garantia permanente. Vale registrarlo: fue el
   propio smoke el que encontro el segundo.
2. **AC-21/AC-22 sin corrida real**: crear un proyecto o un space es una accion
   administrativa irreversible; se dejo implementada y documentada, pero no se
   ejecuto. La primera vez que alguien use `--create-project` conviene mirar el
   resultado.
3. **Paridad ps1 no ejecutada** (sin PowerShell en la maquina), igual que en las
   features #1, #13, #14 y #15: cubierta por lectura y asserts de contenido.
4. **El push de `close` es detached** (OBS-5): el ultimo envio de una feature
   ocurre despues de que el comando devolvio. Si alguien cierra y apaga la
   maquina en el mismo segundo, ese push se aplica en la proxima transicion o
   con `atlassian apply`.
5. **Volumen del backfill**: en un repo grande el primer envio crea muchos
   issues (en este mismo arnes serian ~16 historias y ~240 subtasks). Es la
   decision explicita del usuario (OBS-13/OBS-14: sin umbral, sincronia total);
   el escape documentado es `atlassian backfill --sin-acs`.
6. **Publicar en cada transicion** hace una lectura del space por transicion
   aunque nada cambie (el hash corta antes de escribir). Es barato, pero si
   alguna vez molesta, el lugar para acotarlo es `publish`.
