# Decisiones Pendientes y Aclaraciones de Diseño

Bitácora de decisiones de diseño que el cliente debe revisar/confirmar, o que se documentan
porque no derivan de un RF explícito. Cada entrada indica su estado.

---

## D-01: Claves de acceso fijas (`admin2026`)

**Estado:** Confirmado por el cliente (2026-08-09) — se documenta, no se implementa función de cambio.

**Decisión:**

- La **contraseña genérica de acceso** (login diario, RF-01) y la **clave de configuración
  inicial** (para crear un Bankomunal, RF-03) comparten el mismo valor semilla por defecto:
  `admin2026`, tal como aparece documentado en el manual de usuario.
- El valor **NO está hardcodeado de forma oculta en el binario de Rust como fuente de verdad**:
  solo se usa una vez, como *bootstrap*, en `AuthService::inicializar()` para sembrar el **hash
  bcrypt** en la tabla `app_setting` del `app.db`. A partir de ese momento la verificación del
  login y de la clave de configuración ocurre **siempre contra el hash almacenado en `app.db`**.
- Esto garantiza que la clave vive en la base de datos (persistencia local) y no en una
  constante del código, por lo que en el futuro podría cambiarse **sin recompilar**, actualizando
  el hash en `app.db`.

**Alcance explícitamente descartado (YAGNI):**

- No existe ni se construirá de momento un comando Tauri ni una pantalla para cambiar la
  contraseña o la clave de configuración, porque **ningún RF actual lo solicita** (la contraseña
  es fija y compartida entre la junta y los socios). Si en el futuro el cliente pidiera cambiarla,
  el cambio se hará sustituyendo el hash en `app.db` (o a través de un comando que lo actualice).

**Dónde se implementa:**

- Constante semilla: `src-tauri/src/modules/auth/domain.rs`
  (`DEFAULT_CLAVE_CONFIGURACION`, `DEFAULT_PASSWORD_GENERICA`, valor `"admin2026"`).
- Siembra de hashes: `AuthService::inicializar()` en `src-tauri/src/modules/auth/application.rs`.
- Verificación en runtime: `AuthService::login()` y `AuthService::crear_bankomunal()`
  (leen el hash desde `app.db` vía `AppSettingsPort`).

**⚠ Desactualizado respecto al código (2026-08-12):** hoy `crear_bankomunal()` **no recibe ni
verifica** ninguna clave de configuración; sólo valida nombre no vacío y no duplicado. El test
que la cubría fue eliminado con el comentario *"la clave de configuración ya no existe"*.
Pendiente de decidir: se reimplementa RF-03 con la clave, o se actualiza esta decisión.

---

## D-02: Fórmula del PPCFC (RN-09)

**Estado:** Aceptada de forma tentativa por el cliente (2026-08-12) — **falta que la
organización la corrobore formalmente**.

La primera lectura del cliente (*"el monto que haya ÷ el número de acciones × 100"*) daba
*dinero por acción*, no un porcentaje, y los rangos de RN-09 (<80%, 80-90%, 90-100%) sólo
tienen sentido sobre una razón adimensional. Al plantearlo, el cliente aceptó la lectura
alterna, coherente con el nombre *Promedio de Colocación del Fondo de Crédito*:

```
colocación del mes = cartera ÷ (efectivo + cartera)
PPCFC              = promedio de la colocación de los 3 últimos meses cerrados
```

Es decir: qué proporción del fondo de crédito está efectivamente colocada en préstamos.

**Implementación:** `cierre_mes` guarda `colocacion_pct` y `ppcfc_pct` en cada cierre (v3), de
modo que corregir un cierre no reescriba en silencio la historia del PPCFC. El cálculo vivirá
en una única función de dominio `calcular_ppcfc()` del módulo de Acciones, aislada, para que
corregir la fórmula no toque nada más. Afecta RF-26 y RF-96.

### PENDIENTE — ¿qué pasa en los primeros meses? (a consultar con el cliente)

Un Bankomunal recién creado **no tiene meses cerrados**, así que el PPCFC no se puede calcular.
El formato en papel asume los tres meses (A, B, C) y no dice qué hacer antes; ningún RF lo
cubre. No parece casualidad que RN-02 también arranque "a partir del tercer mes": hay un
período de arranque con reglas distintas que nunca se documentó.

Opciones planteadas, ninguna confirmada todavía:

1. **Venta libre los primeros 3 meses** — sin PPCFC calculable no hay tope de cupo mensual.
   Es lo más coherente con que RN-02 tampoco aplique: el Bankomunal está formando su capital.
2. **Promediar sólo los meses que existan** — con 1 o 2 cierres se promedian ésos y se aplican
   igual los tramos de RN-09.
3. **Autorización manual de la Junta** — el Verificador digita el cupo del mes y queda en
   auditoría, hasta que haya 3 meses cerrados.

Mientras no se resuelva, el módulo de Acciones se construirá con la autorización por PPCFC
aislada tras esta decisión, no cableada al resto de la compra.

---

## D-03: `tope_individual_mensual_pct` no es RN-02

**Estado:** Confirmado por el cliente (2026-08-12).

Son dos topes distintos y ambos aplican:

- **RN-02 — participación acumulada:** ningún socio puede poseer más del **15%** del total de
  acciones del Bankomunal. Se evalúa contra el histórico del socio.
- **`tope_individual_mensual_pct` — tope del cupo del mes:** del cupo que el PPCFC autoriza
  vender ese mes, un solo socio puede tomar como máximo este % (valor por defecto **20%**).

*Ejemplo del cliente:* si el PPCFC autoriza vender un 10% equivalente a $450.000, un socio puede
comprar el 20% de esos $450.000.

El valor por defecto de 20% en el esquema **no era un error**. Se documentó el campo en
`configuracion/domain.rs` y se etiquetó en la pantalla de Configuración para que no se confunda
con RN-02.

---

## D-04: RN-08 son dos parámetros, no uno

**Estado:** Confirmado por el cliente (2026-08-12).

RN-08 mezclaba dos magnitudes que el sistema trataba como una sola:

| Parámetro | Qué es | Defecto |
|---|---|---|
| `pct_fondo_reserva` | % de las ganancias del mes que se retiene al Fondo de Reserva | 10% |
| `tope_fondo_reserva_pct` | Hasta dónde puede crecer el saldo acumulado de la Reserva, como % del capital en acciones | 20% |

**Bug corregido:** `ConfigService::actualizar_configuracion()` rechazaba una retención mensual
mayor al 20%, es decir aplicaba el tope acumulado como límite de la retención mensual,
impidiendo configurar retenciones legítimas. Ahora valida cada uno contra su propio rango.

> El campo se llamó `tope_reserva_incobrables_pct` en la migración v2 y se renombró a
> `tope_fondo_reserva_pct` en la v3, al separarse los tres fondos (ver D-11).

---

## D-05: Fondo de Protección — fuera del alcance

**Estado:** Confirmado por el cliente (2026-08-12).

El Fondo de Protección aplicaba únicamente a los Bankomunales de **Venezuela**; no forma parte
de esta versión. Se retiró del cálculo del Balance de Gestión Mensual Neto (RF-41) y de la
redacción de RF-21 en el Documento de Entendimiento.

**Se conserva** el registro de hasta 2 *protegidos* por socio (tabla `protegido`, RF-21): son
datos de la persona, independientes del fondo que ya no existe. *Pendiente de confirmar: si sin
el fondo el registro de protegidos pierde sentido, se elimina también la tabla.*

---

## D-06: Fórmula del interés de mora (RF-71)

**Estado:** PENDIENTE — el cliente la entrega el 2026-08-13.

No está definido si la mora se calcula como tasa mensual prorrateada por día de atraso, sobre la
cuota vencida o sobre el saldo pendiente, ni si hay un mínimo. El esquema ya tiene lo necesario
para registrarla (`cuota.dias_atraso`, `cuota.mora_pagada`); falta sólo el cálculo.

---

## D-07: La Solicitud de Crédito se persiste como entidad propia

**Estado:** Decidido por el equipo de desarrollo (2026-08-12), a criterio delegado por el cliente.

RF-50 y RF-51 exigen guardar la decisión de la Junta (Aprobado / Modificado / Negado / Diferido)
y mantener las **Diferidas visibles hasta resolverse**. Una solicitud Negada o Diferida nunca
produce un crédito, así que no puede vivir dentro de la tabla `credito`.

Se crearon `solicitud_credito` y `garantia_solicitud` (fiadores propuestos, RF-48). Al aprobar y
desembolsar, los datos se copian a `credito` / `garantia_credito`, y `credito.solicitud_id`
conserva el enlace de origen. RF-44 sigue siendo posible: un desembolso puede registrarse sin
solicitud previa, dejando `solicitud_id` en NULL.

---

## D-08: Vacíos del diccionario de datos corregidos (migración v2)

**Estado:** Aplicado (2026-08-12), autorizado por el cliente.

Al contrastar el esquema v1 contra los RF aparecieron requisitos sin dónde guardarse.
Se corrigieron en `src-tauri/src/core/sql/banco_v2.sql`:

| Vacío | RF | Corrección |
|---|---|---|
| Crédito sin frecuencia de pago ni fecha de vencimiento | RF-60 | `credito.frecuencia_pago`, `credito.fecha_vencimiento` |
| Cuota sólo con `pagada` booleano: no admitía abonos parciales | RF-65, RF-68, RF-72 | `cuota.capital_pagado`, `interes_pagado`, `mora_pagada` |
| Sin tabla de Bienes / Activo Fijo, pese a `cierre_mes.bienes` | RF-88 | tabla `bien` |
| Libro de I/E sin saber si su mes está cerrado ni si fue corregido | RF-89, RF-90, RF-94, RF-97 | `movimiento_libro.cierre_mes_id`, `corregido`, `corregido_por`, `fecha_correccion`, `motivo_correccion` |
| Solicitud de crédito sin persistencia | RF-43, RF-45, RF-48, RF-50, RF-51 | tablas `solicitud_credito`, `garantia_solicitud` (ver D-07) |
| RN-08 con un solo parámetro | RN-08 | `configuracion.tope_fondo_reserva_pct` (ver D-04) |

**Regla adoptada:** `banco_v1.sql` no se edita nunca. Todo cambio de esquema entra como un
script nuevo al final de `BANCO_MIGRATIONS` en `core/db.rs`, que ahora aplica las migraciones
pendientes en orden y **cada una dentro de una transacción** (si falla, la base queda en su
versión anterior íntegra, no a medio migrar). Cubierto por el test
`migracion_v1_a_v2_conserva_los_datos`.

---

## D-09: Los reportes se exportan a Excel

**Estado:** Confirmado por el cliente (2026-08-12).

RF-107 se implementará exportando a **Excel (.xlsx)**, no a PDF. Se generará desde Rust
(candidato: crate `rust_xlsxwriter`, sin dependencias nativas ni Office instalado), guardando el
archivo en la ruta que elija el usuario. La impresión queda delegada a Excel.

---

## D-10: Respaldo cifrado y verificable

**Estado:** Confirmado a medias (2026-08-12) — **falta precisar el formato**.

El cliente confirmó que el respaldo debe ir **cifrado** (recomendación de la sección 7 del
documento de Arquitectura: el `.db` lleva cédulas e información financiera y suele viajar en
PenDrive).

Diseño propuesto, a confirmar antes de implementar: el respaldo no es el `.db` crudo sino un
archivo empaquetado que incluye el `.db` más un manifiesto con `id` y `nombre` del Bankomunal,
fecha y versión de esquema. El manifiesto es lo que permite cumplir RF-111 (validar que el
archivo corresponda al Bankomunal correcto) antes de sobrescribir nada.

**Pendiente:** confirmar si el cifrado lleva contraseña propia que el usuario escribe al
respaldar/restaurar, o se deriva de la contraseña genérica de la app.

---

## D-11: Los fondos son dos

**Estado:** Confirmado por el cliente con explicación detallada y ejemplo numérico
(2026-08-12). **Corrige una respuesta anterior del mismo día que decía "son tres".**

| Fondo | Para qué sirve | Cómo se llena | Cómo se gasta |
|---|---|---|---|
| **Fondo para Gastos** (RN-07) | Gastos operativos del día a día: fotocopias de formatos, transporte de un miembro de la Junta, papelería, una calculadora nueva | % de las ganancias del mes (por defecto 10%), automático en el Cierre; más las donaciones (RF-87) | Gastos del Bankomunal (RF-86) |
| **Fondo de Reserva para Incobrables** (RN-08) | Colchón de seguridad para cuando un socio no puede pagar su deuda y sus acciones no alcanzan a cubrirla | % de las ganancias del mes (por defecto 10%), con **tope del 20% del capital total en acciones**: al llegar al tope deja de crecer, ya es colchón suficiente | Se descuenta al absorber un incobrable (RF-36) |

En una frase: **Gastos** = plata para que el Bankomunal funcione. **Incobrables** = plata
guardada por si alguien no paga.

Ejemplo del cliente — el Bankomunal ganó $100.000 en intereses este mes:

```
$10.000 → Fondo de Gastos
$10.000 → Fondo de Reserva para Incobrables
$80.000 → se reparte entre los socios como ganancia
```

Si nadie deja deuda sin pagar, el fondo de Incobrables nunca se usa: sólo crece mes a mes
hasta su tope. Sólo se consume cuando ocurre de verdad una liquidación con deuda mayor al
valor de las acciones del socio.

**Historia de esta decisión.** Una respuesta breve del cliente ("son 3: gastos, reserva e
incobrables") llevó a separar un tercer fondo en la migración v3. Su explicación detallada
posterior usa *"Fondo de Reserva para Incobrables"* y *"Fondo de Incobrables"* como sinónimos
del mismo fondo, coincide con la redacción literal de RN-07/RN-08 y con la pantalla de
Configuración que ya existía. **La migración v4 revierte el tercer fondo**, consolidando
defensivamente cualquier saldo que hubiera quedado en él.

**Sigue pendiente:** ¿qué ocurre cuando la Reserva no alcanza para absorber un incobrable?
Ningún RF lo define.

---

## D-12: Ganancias — devengo mensual, pago al año

**Estado:** Confirmado por el cliente (2026-08-12). Resuelve la ambigüedad entre RN-10 y
RF-39/RF-42.

Las ganancias **no** se pagan cada mes ni se calculan con un valor fijo de configuración.
El modelo real es de **acumulación**:

1. Cada mes, al cerrar, el sistema calcula el **valor de ganancia por acción** de ese mes
   (Balance de Gestión Mensual Neto ÷ acciones activas).
2. Cada lote de acciones activo **devenga** `cantidad × valor por acción` de ese mes. El valor
   queda guardado; no se paga todavía.
3. Eso se repite mes a mes hasta que el lote **cumple un año** desde su compra (RN-10), momento
   en que la ganancia acumulada se paga al socio.

*Ejemplo del cliente:* si Pepito compró 5 acciones y la ganancia de ese mes fue de $1.000 por
acción, ese mes devenga $5.000; y así sucesivamente hasta cumplir el año.

**Implementado en la migración v3.** `pago_ganancia` pasa a ser el renglón de devengo mensual
por lote:

- `fecha_pago` admite **NULL** — antes era NOT NULL, lo que hacía imposible representar una
  ganancia devengada pero aún no pagada. Requirió reconstruir la tabla (SQLite no permite
  alterar la nulabilidad de una columna).
- Se guardan `acciones` y `valor_por_accion` como fotografía del mes, para que corregir un
  cierre posterior no altere el histórico ya devengado.
- `UNIQUE (lote_acciones_id, reparto_ganancias_id)` impide devengar dos veces el mismo lote en
  el mismo mes — la protección más importante del modelo, porque el devengo se ejecuta en cada
  cierre y un cierre repetido duplicaría el dinero.
- `estado`: `PENDIENTE` (devengada) → `PAGADA` (liquidada al cumplir el año).

> La pregunta que quedó abierta aquí —qué pasa si el socio liquida antes del año— quedó
> resuelta en **D-13**.

---

## D-13: Ganancia no consolidada al liquidar antes del año

**Estado:** Confirmado por el cliente (2026-08-12). Afecta RF-31, RF-33, RF-34, RF-35 y RF-37.

El reglamento dice: *"Las ganancias de las acciones se repartirán al año de vencida cada
acción"*. Es una regla de **todo o nada**, como un CDT que sólo paga intereses si se deja
cumplir el plazo: no es una ganancia que se acumule proporcionalmente y se pueda cobrar a
medias. Si el lote no cumple su año, todavía no genera derecho a esa ganancia.

**Regla:**

1. Al liquidar acciones que **no** han cumplido su año, se devuelve únicamente el **valor
   nominal invertido** (el capital). La ganancia del año en curso no se paga.
2. Las ganancias de **años anteriores ya pagadas** no se ven afectadas — son plata entregada
   hace tiempo, aparte, y no se recuperan ni se recalculan.
3. La ganancia no pagada del año en curso **no desaparece del sistema**. El dinero ya está
   físicamente en la caja, generado mes a mes; simplemente nunca se le asignó a un socio.
   Queda dentro de *"ganancias no repartidas"* en el Pasivo, beneficiando colectivamente al
   Bankomunal en el próximo cierre.

**Cálculo del "valor a favor del socio" en Liquidación (RF-31):**

```
valor a favor = capital nominal de las acciones liquidadas
              + ganancias de años ya cumplidos que estén devengadas y aún sin pagar
```

Nunca incluye la ganancia del año corriente sin cumplir.

> **Matiz que conviene confirmar.** El cliente escribió *"capital + ganancias ya efectivamente
> pagadas anteriormente"*. Sumar una ganancia **ya pagada** implicaría pagarla dos veces, así
> que se interpretó como *"esas no se tocan"* (punto 2 de la regla). Lo que sí debe sumarse es
> la ganancia de un año **ya cumplido** que todavía no se haya entregado —por ejemplo, si el
> lote cumple su aniversario y el socio liquida antes de que se ejecute el reparto—.

**Implementado en la migración v4.** `pago_ganancia.estado` admite un tercer valor:

| Estado | Significado |
|---|---|
| `PENDIENTE` | Ganancia de un mes ya devengada y todavía no entregada al socio |
| `PAGADA` | Entregada al socio al cumplir el lote su aniversario |
| `LIBERADA` | El lote se liquidó antes del aniversario; queda para el colectivo |

*Ganancias no repartidas* del Pasivo = suma de `PENDIENTE` + `LIBERADA`. Se agregaron
`pago_ganancia.fecha_liberacion` y `lote_acciones.fecha_liquidacion`, esta última para decidir
qué renglones se liberan y comprobar si el lote alcanzó su aniversario.

### Cuándo se crea el renglón de devengo (aclaración)

La definición inicial de `PENDIENTE` decía *"el lote aún no cumple el año"*, lo cual dejaba sin
cubrir el caso que motivó esta decisión: un lote que **ya cumplió** su año pero cuyo reparto
formal todavía no se ha ejecutado. La definición correcta es la de la tabla de arriba —
*devengada y no entregada*— y **no hace falta un cuarto estado**, porque el aniversario no es un
atributo del renglón: se deduce comparando `lote_acciones.mes_compra` con la fecha del reparto.

El flujo es **crear el renglón en el Cierre de Mes, nunca al liquidar**:

1. En cada **Cierre de Mes** se calcula el valor de ganancia por acción y se inserta un renglón
   `PENDIENTE` por cada lote activo. El devengo es un efecto del cierre, no de la liquidación.
2. Al **liquidar**, no se calcula ni se crea nada al vuelo: sólo se leen los renglones que ya
   existen y se clasifican por la antigüedad del lote —
   `SUM(monto) WHERE estado = 'PENDIENTE'`, separando los meses anteriores al aniversario
   (se pagan) de los del año en curso (se liberan).

Se descartó calcular el monto al vuelo en el momento de liquidar: obligaría a reconstruir el
balance neto de meses ya cerrados, y una corrección posterior a un cierre (RF-90) haría que la
misma acción valiera distinto según cuándo se consulte. Guardar el devengo en el cierre deja el
histórico inmutable, que es justo para lo que existen las columnas `acciones` y
`valor_por_accion`.

> **Consecuencia operativa:** un lote comprado y liquidado dentro del mismo mes, antes de que
> corra el cierre, no tiene ningún renglón — no alcanzó a devengar nada. Es correcto: sólo se
> le devuelve el capital.

---

## D-14: PPCFC — umbrales fijos vs. % autorizado a vender

**Estado:** Corregido a partir del formato original *"Reporte de acciones autorizadas para la
venta"* (2026-08-12).

El formato en papel deja el cálculo explícito:

```
A, B, C = % de colocación de crédito de los 3 meses
D       = A + B + C
E       = D ÷ 3            ← el PPCFC
```

y luego los tramos: `E < 80%` → no se venden acciones; `80% ≤ E < 90%` → hasta **10%** del total
de acciones; `90% ≤ E ≤ 100%` → hasta **15%**. Esto confirma la fórmula de D-02.

**Bug corregido.** Las columnas `ppcfc_rango1_pct`/`ppcfc_rango2_pct` guardaban **80 y 90** —los
umbrales— mientras la pantalla de Configuración las presentaba y editaba como *"% autorizado a
vender"*. Cualquiera que hubiera guardado la configuración habría autorizado a vender el 80% del
Bankomunal a un solo socio.

Son dos cosas distintas y se separaron:

- **Umbrales (80 / 90 / 100):** vienen del reglamento **fijo** de la metodología, iguales para
  todos los Bankomunales. Pasan a ser constantes de dominio (`PPCFC_UMBRAL_MINIMO`,
  `PPCFC_UMBRAL_MEDIO`), fuera de la tabla.
- **% autorizado a vender (10 / 15):** es lo configurable. Columnas renombradas a
  `ppcfc_venta_rango1_pct` / `ppcfc_venta_rango2_pct`, con los valores por defecto corregidos
  (migración v5, que sólo corrige las filas que aún tenían el valor sembrado erróneo).

---

## D-15: Correcciones desde los documentos originales del cliente

**Estado:** Aplicado (2026-08-12), a partir del reglamento fijo, la planilla de registro del
socio, el catálogo de clases de crédito y los formatos en Excel.

**a) RN-02 aplica desde el tercer mes.** El texto completo del reglamento es: *"Ningún socio
podrá poseer más del 15% del total de las acciones... **Esta regla aplica a partir del tercer
mes de iniciadas las operaciones**"*. La condición temporal no estaba en el documento de
entendimiento. Tiene sentido: en el mes 1, con 8 socios, cada uno pasa del 15% por aritmética
pura, y sin la excepción sería imposible arrancar un Bankomunal.

**b) RN-11 — catálogo de clases de crédito corregido.** El documento listaba
`AH, ARE, CV, CRV, ED, GP, PR, SL, Servicios Públicos, Otros`. El catálogo oficial es:

| | | | | |
|---|---|---|---|---|
| **AH** Adquisición Artículos para el Hogar | **ARE** Adquisición o Reparación de Equipos de Trabajo | **CVR** Construcción o Reparación de Vivienda | **CV** Compra Venta de mercancías | **ED** Educación |
| **GP** Gastos Personales | **OT** Otros | **PR** Productivos | **SL** Salud | **SP** Servicios Públicos |

Diferencias: `CRV` era una errata de **CVR**, y "Servicios Públicos"/"Otros" tienen código
propio (**SP**, **OT**).

**c) Fondos: mínimo 5%.** *"Se apartará un fondo de gastos, no menor al 5% de las ganancias
totales de cada mes"* (ídem incobrables). El mínimo no es 0: ya se valida.

**d) Regla que falta como RN.** *"Todo crédito debe tener al menos un fiador que debe ser socio
del Bankomunal"*. Nuestros RF sólo dicen "hasta 2 fiadores" (RF-48), sin exigir el mínimo de 1.
Habrá que validarlo en el módulo de Créditos.

**e) Datos del socio (migración v5).** La planilla pide **teléfono** de cada protegido, que
faltaba en la tabla, y para el beneficiario en caso de muerte sólo pide nombre y cédula, así que
`beneficiario.parentesco` pasó a ser opcional.

**f) Valor de la acción.** El reglamento recomienda **$10.000**; el valor por defecto del sistema
es $20.000. Se deja como está por ser configurable (RN-13), pero conviene confirmarlo.

**Reglas del reglamento que NO se implementan** (no las pide ningún RF; se registran para que la
decisión sea consciente): las operaciones sólo ocurren en la Reunión de Socios; todo socio debe
ejercer una función de la Junta al menos una vez al año; todo socio está obligado a solicitar al
menos un crédito al año; edad mínima y tiempo de permanencia en la comunidad (reglas variables
que hoy no se capturan en ningún formulario).

---

## D-17: Reglas de negocio formalizadas (RN-14 y RN-15)

**Estado:** Confirmado por el cliente (2026-08-13).

Dos reglas que existían en los documentos del cliente pero nunca se escribieron en el
Documento de Entendimiento. Ya están agregadas:

**RN-14 — Mínimo 1 fiador.** *"Todo crédito debe tener al menos un fiador que debe ser socio
del Bankomunal"* (reglamento fijo). Los RF sólo decían "hasta 2 fiadores" (RF-48), sin exigir
el mínimo. Es regla fija, no opcional. **Se validará en el módulo de Créditos**, junto a RN-04
(garantía del 40%) y RN-05 (no fiadores cruzados).

**RN-15 — Tope individual del cupo mensual.** Ningún socio puede comprar, en un mismo mes, más
del **20% (configurable)** del cupo que el PPCFC ya autorizó vender ese mes. El porcentaje se
calcula sobre **el cupo del mes**, no sobre el capital total del Bankomunal. Fuente: documento
*"Cargos Junta Administradora"*.

Es el campo `tope_individual_mensual_pct`, que ya existía en Configuración pero estaba
huérfano de regla (se documentó provisionalmente como D-03). Queda reconectado en el dominio,
en la validación y en la pantalla de Configuración; **se aplicará en la pantalla de Compra de
Acciones** cuando exista el módulo (RF-22 a RF-27).

> RN-15 y RN-02 son topes distintos y **ambos aplican**: RN-15 mira el cupo del mes, RN-02 mira
> la participación acumulada (15% del total de acciones, desde el tercer mes — ver D-15).

Con esto la numeración llega a **RN-01 … RN-15**, lo que además resuelve la referencia a
"RN-01 a RN-14" del documento de Arquitectura, que apuntaba a reglas que no existían.

---

## D-18: Validación de formato en los formularios

**Estado:** Implementado (2026-08-13), a petición del cliente.

- **Correo:** debe tener usuario, una sola `@`, dominio con punto y sin espacios. No se usa una
  expresión del RFC completo a propósito: las direcciones exóticas que rechazaría no existen en
  este contexto y el patrón sería ilegible de mantener.
- **Campos numéricos** (cédula, teléfono, celular, y los mismos de beneficiario y protegidos):
  sólo dígitos. En pantalla se descarta el carácter no numérico **mientras se teclea**, en vez
  de dejar escribir y avisar al guardar.

**Se valida en los dos lados.** El componente `Campo` lo hace en el formulario, para que el
usuario lo vea al instante; y `SocioService::validar()` lo repite en Rust, porque el Dominio no
puede asumir que esa pantalla sea la única entrada posible (un respaldo restaurado, un comando
invocado de otra forma). Es duplicación deliberada, no un descuido.

**Consecuencia a vigilar:** sólo se aceptan dígitos, sin guiones ni espacios. Sirve para la
cédula colombiana; si algún día se registran documentos con otro formato, hay que relajar la
regla en los dos lados.

---

## D-19: Compra de Acciones — decisiones de implementación

**Estado:** Implementado (2026-08-13). Cubre RF-22, RF-23, RF-24, RF-25 y RF-27.
**RF-26 (autorización por PPCFC) queda fuera** hasta resolver D-02.

**a) El lote guarda el valor nominal con el que se compró.** RN-13 permite que la asamblea
cambie el valor de la acción. Si el lote sólo guardara la cantidad, al liquidar habría que
multiplicar por el nominal *actual* y el socio recibiría más —o menos— de lo que puso. Guardar
`valor_nominal_compra` y `monto_pagado` congela el capital aportado, que es justo lo que D-13
manda devolver. Migración v6.

**b) El monto debe ser múltiplo exacto del valor nominal.** Las acciones son unidades enteras.
Aceptar $25.000 con nominal de $10.000 obligaría a quedarse con $5.000 del socio o a regalar
media acción. El sistema rechaza el monto y responde con los dos montos válidos más cercanos
("$20.000 compra 2 acciones y $30.000 compra 3"), para que el Cajero corrija en el acto.
*A confirmar: si en la práctica reciben montos libres y devuelven el sobrante, habría que
registrar esa devolución como egreso.*

**c) RN-02 se evalúa con su período de gracia, y sin él sería imposible arrancar.** El primer
socio de un Bankomunal nuevo tiene por definición el 100% de las acciones. La excepción del
tercer mes que descubrimos en el reglamento (D-15) no es un detalle menor: es lo que permite
formar el capital inicial. El sistema calcula los meses transcurridos desde
`configuracion.fecha_creacion` y sólo aplica el tope a partir del tercero.

> Esto apareció al escribir las pruebas: el primer test de compra fallaba con "el socio quedaría
> con el 100%". No era un fallo del test sino la regla funcionando; el montaje de prueba era el
> irreal, porque creaba un Bankomunal "de hace cinco años" con cero acciones.

**d) El tope permite exactamente el 15%.** RN-02 dice "más del 15%", así que 15,00% clavado se
acepta y 15,01% se rechaza.

**e) Acciones no escribe en el Libro por su cuenta.** El consecutivo, el saldo acumulado y su
recálculo son responsabilidad de Caja. Se agregó `CajaService::registrar_asiento_de_modulo()`,
que no se expone como comando Tauri y que rechaza los códigos que sí se teclean desde Caja.
Duplicar la lógica del Libro en cada módulo sería la vía más corta a un libro descuadrado.

**f) `Socio::acciones_activas()` deja de estar pendiente.** La pregunta se responde con los
lotes, así que vive en `AccionesService`, no en la entidad `Socio`. Siguen pendientes
`acciones_libres()` y `tiene_credito_vigente()`, que dependen de Créditos.

**Lo que este módulo todavía NO hace**, y por qué:

**g) El PPCFC y el cupo del mes SÍ están implementados (RF-26, RN-09, RN-15).** La fórmula está
confirmada, así que se programó completa: `calcular_ppcfc()` promedia la colocación de los 3
últimos cierres, `tramo_de_venta()` aplica los umbrales del reglamento y el servicio deriva el
cupo en acciones y en dinero, descontando lo ya vendido en el mes.

Lo único que sigue abierto es **qué hacer cuando faltan meses cerrados**, y ahí el sistema
devuelve `SinDatosSuficientes` y **no bloquea la venta**: bloquear equivaldría a elegir por el
cliente una de las tres opciones de D-02. La pantalla lo muestra como pendiente y deja la
autorización a criterio de la Junta. Cuando exista el Cierre Mensual —que es quien sella
`cierre_mes.colocacion_pct`— el panel empieza a calcular solo, sin tocar este módulo.

| Falta | Depende de |
|---|---|
| Qué hacer con el PPCFC en los primeros meses | D-02 (decisión del cliente) |
| RF-28 a RF-38: Liquidación de Acciones | Créditos — los tres escenarios de deuda (RF-32 a RF-36) son el corazón del caso de uso y sin deuda quedaría una función a medias que parece completa |
| RF-39 a RF-42: Ganancias Repartidas | Cierre Mensual, que produce el Balance de Gestión Neto del que sale el valor por acción (D-12) |

---

## D-16: Alcance del módulo de Socios

**Estado:** Implementado (2026-08-12).

**Campos obligatorios.** Sólo se exigen **cédula, nombres y apellidos**. Profesión, dirección,
teléfono, celular y correo quedan opcionales aunque RF-17 los enumere: los socios son población
rural y muchos no tienen correo electrónico, así que obligarlos llevaría a llenar el campo con
datos falsos, que es peor que dejarlo vacío (RNF-04). *A confirmar con el cliente.*

**RN-01 al registrar.** El máximo de 19 socios se valida como tope duro. El mínimo de 8 no se
puede exigir —un Bankomunal arranca en cero y va sumando— así que se expone como información en
pantalla, no como bloqueo. Sólo cuentan los socios **activos**: un socio retirado libera su cupo.

**Métodos que quedan pendientes.** `acciones_libres()`, `acciones_activas()` y
`tiene_credito_vigente()` están declarados en `Socio` con `todo!()`, sin lógica provisional y sin
que ningún comando Tauri los invoque. Devolver 0 o el total habría sido una respuesta plausible
pero falsa, y las Liquidaciones se calcularían mal sin que nadie lo note.