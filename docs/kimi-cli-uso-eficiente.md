# Uso eficiente de Kimi Code CLI (multiagente)

Guia del arnes para trabajar con Kimi Code CLI gastando la minima cantidad de
tokens posible. Aplica a cualquier proyecto donde el arnes este instalado; el
instalador (`setup_harness.sh` / `setup_harness.ps1`) ya siembra los archivos
base sin pisar los existentes.

## Paso 1: exclusiones de contexto (`.gitignore` + `.kimiignore`)

Las dependencias, credenciales, dumps y logs son la mayor fuente de desperdicio
de tokens: si el agente los indexa o los busca, paga ese costo en cada comando.

- El instalador siembra `.kimiignore` en la raiz del proyecto (no pisa uno
  existente). Copia esas mismas lineas a tu `.gitignore` y manten ambos
  archivos sincronizados: Kimi Code ya respeta `.gitignore` en sus busquedas, y
  `.kimiignore` deja las exclusiones explicitas para cualquier agente o
  herramienta que lo lea.
- Ajusta las rutas a tu stack. Ejemplo para un proyecto Laravel/PHP:

```text
# Dependencias masivas (bloqueo critico de tokens)
/vendor/
/node_modules/
/public/build/

# Entornos y credenciales
.env
.env.backup
.phpunit.result.cache

# Docker y bases de datos
.docker/
/mysql_data/
/*.sql

# Logs y cache local
/storage/*.log
/bootstrap/cache/
.zed/
.kimi/
```

## Paso 2: reglas fijas del proyecto (`.kimirules`)

Para no repetir en cada comando el contexto de dominio (moneda, identificadores
tributarios, invariantes de negocio), escribelo UNA vez en `.kimirules` en la
raiz. El instalador siembra la plantilla; el `AGENTS.md` de la raiz (que Kimi
Code lee automaticamente al abrir la sesion) apunta a ese archivo, asi que las
reglas entran como contexto fijo de cada sesion sin reescribirlas.

- Que va ahi: lo que NO cambia entre comandos. Ejemplo (SaaS inmobiliario
  chileno): pesos chilenos (CLP, montos enteros), RUT unico con digito
  verificador, IVA 19% y logica de facturacion del SII, roles
  broker/owner/tenant.
- Que NO va ahi: instrucciones de una tarea puntual (eso es el prompt) ni
  estado volatil (eso vive en `progress/`).

## Paso 3: mecanica de comandos eficientes

Nunca lances preguntas genericas: obligan al agente a adivinar alcance y a
generar codigo de mas (tokens de salida que no querias). Usa **acotamiento por
archivo**: ruta exacta + cambio concreto.

- Ineficiente (gasta saldo a ciegas):
  `kimi "haz la base de datos de los usuarios con los roles de la reunion"`
- Eficiente (un solo archivo, contexto ya en `.kimirules`):
  `kimi "Genera la migracion database/migrations/create_users_table.php con el enum de roles broker/owner/tenant y el campo rut unico"`

## Paso 4: boton de reinicio

La sesion mantiene el historial completo y cada comando nuevo lo reenvia
entero: una hora de trabajo en la misma sesion multiplica los tokens de entrada
de todo lo que sigue.

- En cuanto una tarea queda terminada y verificada (por ejemplo, la migracion
  ya escrita y visible en tu editor), ejecuta `/new` (alias `/clear`) antes de
  la siguiente.
- Retoma con una referencia al artefacto, no al historial:
  `kimi "Tomando como base la migracion existente, genera el Modelo User con sus fillables"`
  El agente lee el archivo nuevo y `.kimirules`, y pagas el minimo.

## Resumen

1. `.kimiignore` + `.gitignore` sincronizados: fuera dependencias, secretos,
   dumps y logs.
2. `.kimirules`: dominio fijo escrito una sola vez.
3. Comandos acotados por archivo: ruta exacta + cambio concreto.
4. `/new` entre tareas: el historial viejo se paga en cada comando.
