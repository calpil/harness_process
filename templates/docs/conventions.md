# Convenciones

- Usa Conventional Commits.
- No agregues `Co-Authored-By` ni firmas generadas por IA.
- Trabaja dentro del microservicio afectado.
- Prefiere cambios pequenos, verificables y documentados.
- Registra decisiones relevantes en `progress/`.

## La escalera de huella

Cada peldano deja mas superficie permanente que el anterior. Elegi siempre **el
peldano de menor huella que resuelva el problema** — no el que te resulte mas
comodo de escribir.

1. **Extender lo que ya existe** — cero superficie nueva (#24). La capacidad es
   una variacion de algo que ya esta. *Aplica cuando* el comportamiento nuevo
   cabe dentro de un modulo o un documento existente sin cambiarle el contrato.
   Ejemplo: el bloque de conventions vive dentro de `harness_check.sh`, en vez de
   un comando `harness_cli conventions` que sumaria superficie permanente para
   leer un markdown.
2. **Flag en un comando existente** — el comando ya hace lo correcto (#21).
   *Aplica cuando* la diferencia es un parametro, no un flujo. Ejemplo:
   `--aplicar` en `lecciones curar` y `--solo AC-n` en `verify` son dos modos del
   mismo comando, no dos comandos.
3. **Comando nuevo** — hay un verbo propio, con su exit code y su salida (#20).
   *Aplica cuando* el usuario lo va a invocar por su cuenta y no es un modo de
   otra cosa. Ejemplo: `buscar` y `journey` responden preguntas distintas sobre
   datos distintos; ninguno de los dos cabia como flag del otro.
4. **Superficie nueva** — archivo generado, hook o artefacto versionado (#17).
   *Aplica cuando* la informacion tiene que sobrevivir a la sesion y ser leida
   por otros. Ejemplo: `docs/lecciones/` y `docs/perfil-usuario.md` existen para
   durar mas que cualquier conversacion, y por eso son archivos del repo y no
   filas en una base.
5. **Dependencia nueva** — ultimo recurso, y exige ADR (#15). Lo pide el
   Articulo 6 de `docs/constitution.md`. *Aplica cuando* nada de lo anterior
   alcanza y mantenerla cuesta menos que escribirla. Ejemplo: `ureq` entro con
   `ADR-0001` y alternativas evaluadas. Contraejemplo del peldano: el timeout de
   `verify` reuso `wait-timeout`, que ya era dependencia, y por eso no necesito
   ADR.

### Si no tomas el peldano mas alto, escribilo

Cuando elijas un peldano que **no** es el de menor huella posible, el plan lo
declara con esta linea exacta, para que el reviewer la pueda buscar:

```
Peldano elegido: <n> (<nombre>) porque <razon concreta>
```

La razon tiene que decir por que el peldano de mas arriba **no alcanzaba**, no
por que el elegido es agradable. "Es mas claro asi" no es una razon; "un flag
obligaria a que el comando tenga dos exit codes incompatibles" si.

## Reglas de test

Tres reglas: **contratos de comportamiento** y no snapshots, prohibido **leer el
codigo fuente** en un test, y prohibido el **detector-de-cambios**. Las tres
nacieron de casos reales de este repo, y el reviewer **rechaza** el codigo que
las viola.

### 1. Contratos de comportamiento, no snapshots

Un test tiene que asserta como se relacionan dos cosas (un invariante), no
congelar el valor de hoy.

```rust
// NO: congela un dato que se espera que crezca
assert_eq!(fuentes().len(), 12);
assert_eq!(VERSION, "0.3.0");

// SI: el invariante que de verdad importa
assert!(fuentes().len() >= 1, "el catalogo quedo vacio");
assert!(fuentes().iter().all(|f| f.peso() > 0), "toda fuente pesa");
```

La pregunta que decide: *cuando este dato cambie por una razon legitima, ¿el test
tendria que fallar?* Si la respuesta es no, no lo congeles.

### 2. Prohibido leer el codigo fuente en un test

Un test que lee el texto de un archivo fuente prueba **la forma del codigo**, no
su comportamiento. Falla ante un refactor correcto y pasa cuando la
implementacion esta sutilmente rota: las dos direcciones del error.

```rust
// NO: el test de la #23. Grepeaba src/**/*.rs buscando "verify::run".
//     Pasaba aunque verify estuviera mal cableado, y fallaba si alguien
//     renombraba la funcion sin cambiar nada del comportamiento.
let texto = std::fs::read_to_string("src/commands/close.rs")?;
assert!(!texto.contains("verify::run"));

// SI: el mismo criterio, como comportamiento observable (#24).
//     El spec declara `Comando: touch rastro.txt`; se corren los comandos
//     del arnes y se mira el disco.
correr(&["close", "--feature", "1", "--status", "done"]);
assert!(!sandbox.join("rastro.txt").exists(), "close ejecuto el comando");
```

**La unica excepcion admitida**: leer un archivo que es **dato de entrada** del
codigo bajo prueba. Los specs que parsea `verificacion.rs`, las plantillas que
siembra el instalador, un corpus de fixtures: eso no es leer el fuente, es
alimentar al codigo con lo que va a recibir en produccion.

El corte, cuando dudes, es esta pregunta:

> ¿el test seguiria valiendo si la implementacion **se reescribiera entera**?

Si sobrevive esa pregunta, es dato de entrada. Si el test depende de como esta
escrito el codigo hoy, es un test de forma y sobra.

### 3. Prohibido el test detector-de-cambios

Un test es detector-de-cambios si falla cada vez que se actualiza un dato que
**se espera que cambie**: catalogos, numeros de version, conteos de
enumeraciones. No agregan cobertura; solo garantizan que una actualizacion
rutinaria rompa CI y alguien pierda el rato "arreglandolo".

```rust
// NO: se rompe cada vez que alguien escribe una leccion
assert_eq!(lecciones_activas().len(), 7);

// SI: la relacion que tiene que valer siempre
for l in lecciones_activas() {
    assert!(!l.triggers.is_empty(), "{}: sin triggers no se encuentra", l.nombre);
}
```

Si el test se lee como una foto del estado actual, borralo. Si se lee como un
contrato sobre como se relacionan dos datos, quedatelo.

### El chequeo automatico

`harness_check.sh` avisa cuando un test lee un archivo fuente, con archivo, linea
y nombre del test. **Avisa y no bloquea** (`[i]`), porque la regla admite la
excepcion de dato de entrada y un gate duro empujaria a inventar un `--force`,
que es peor que el aviso. Las otras dos reglas no se chequean solas: entender que
dato "se espera que cambie" no se grepea, y las verifica el reviewer.
