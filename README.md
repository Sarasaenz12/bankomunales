# Bankomunales

Aplicación de escritorio para administrar **Bankomunales**: bancos comunitarios donde un grupo de socios ahorra comprando acciones y se presta dinero entre sí, con las ganancias repartidas cada mes entre todos.

El sistema lleva la contabilidad completa del Bankomunal: socios, acciones, créditos, pagos y mora, caja, cierre de mes, reparto de ganancias, reportes y respaldos. Funciona **sin internet**, instalado en el computador de quien administra el banco, y un mismo equipo puede manejar varios Bankomunales por separado.

---

## Contenido

- [Qué hace](#qué-hace)
- [Cómo funciona](#cómo-funciona)
- [Instalación (usuario final)](#instalación-usuario-final)
- [Instalación para desarrollo](#instalación-para-desarrollo)
- [Generar el instalador](#generar-el-instalador)
- [Estructura del proyecto](#estructura-del-proyecto)
- [Documentación](#documentación)

---

## Qué hace

| Módulo | Para qué sirve |
|---|---|
| **Ingreso y selección** | Entrada con contraseña. Se elige el Bankomunal con el que se va a trabajar o se crea uno nuevo. |
| **Inicio (Dashboard)** | Resumen del banco: efectivo en caja, cartera de créditos, socios activos, fondos y alertas. |
| **Datos del Bankomunal** | Parámetros del reglamento: valor de la acción, tasas de interés y mora, garantías, plazos, montos máximos y porcentajes de los fondos. Los cambios quedan en la auditoría. |
| **Socios** | Registro de socios con beneficiario y personas protegidas. Cada socio recibe un número correlativo que no se reutiliza. Incluye retiro y liquidación. |
| **Acciones** | Compra de acciones, venta parcial, historial por socio y cupo de venta del mes según el **PPCFC**. |
| **Créditos** | Solicitud, aprobación por la Junta, desembolso con fiadores, tabla de amortización y refinanciamiento. |
| **Pagos y mora** | Pago de una o varias cuotas, abonos a capital (re-amortizan el crédito) y cálculo de mora. |
| **Caja y Contabilidad** | Libro de ingresos y egresos, donaciones, gastos, bienes del banco, fondo de gastos y fondo de incobrables. |
| **Cuadre y Cierre de Mes** | Balance (Activo = Pasivo), cierre del mes, reparto de ganancias por acción e historial de cierres. |
| **Reportes** | Socios, acciones, créditos, fiadores, libro, balance e informe de gestión, exportables a Excel. |
| **Respaldo y Restauración** | Copia de seguridad del banco en un archivo y restauración desde ese archivo. |
| **Auditoría** | Registro de quién cambió qué y cuándo (configuración, correcciones, cierres). |

### Reglas principales del reglamento

Estos son los valores por defecto. Cada Bankomunal puede ajustarlos en **Datos del Bankomunal**.

- **Socios:** máximo **19 socios activos**.
- **Acciones:** valor nominal de **$10.000**. Ningún socio puede tener más del **15 %** de las acciones del banco.
- **Créditos:** hasta **$5.000.000** y **36 cuotas**, con interés ordinario del **3 % mensual** y mora del **5 %**. Hay tres límites:
  - El monto no puede superar **5 veces** el valor de las acciones del socio.
  - El titular debe garantizar el **20 %** del crédito con sus acciones.
  - Los fiadores, que deben ser socios, garantizan otro **20 %**. No se permiten fiadores cruzados.
- **Mora:** no se cobra dentro del mes de vencimiento. Corre desde el día 1 del mes siguiente.
- **Fondos:** cada mes se retiene un **10 %** para el fondo de gastos y un **10 %** para el de incobrables.
- **PPCFC:** es el promedio de colocación de los 3 últimos cierres. Por debajo del 80 % no se venden acciones en el mes. Entre el 80 % y el 90 % se puede vender hasta el 10 % del total de acciones, y desde el 90 %, hasta el 15 %. Ningún socio puede comprar más del 20 % de ese cupo.
- **Dinero:** todos los valores se manejan en pesos enteros, sin centavos.

---

## Cómo funciona

```
┌──────────────────────────────┐        ┌─────────────────────────────────────┐
│  Interfaz (React + TypeScript)│  IPC   │  Núcleo (Rust)                      │
│  pantallas, formularios,      │ ─────► │  reglas de negocio por módulo       │
│  reportes                     │ ◄───── │  (socios, acciones, créditos, ...)  │
└──────────────────────────────┘        └──────────────────┬──────────────────┘
                                                           │
                                         ┌─────────────────▼──────────────────┐
                                         │  SQLite                             │
                                         │  app.db  → catálogo de bancos       │
                                         │  <id>.db → un archivo por Bankomunal│
                                         └─────────────────────────────────────┘
```

- **[Tauri 2](https://tauri.app/)** empaqueta la aplicación como un programa de Windows liviano.
- **Interfaz:** React 19 + TypeScript + Vite. Solo muestra datos y captura formularios. Todas las reglas viven en el núcleo.
- **Núcleo:** Rust con arquitectura hexagonal. Cada módulo (`src-tauri/src/modules/*`) tiene sus capas `domain` (reglas), `application` (casos de uso), `data` (SQLite) y `presentation` (comandos que llama la interfaz).
- **Base de datos:** SQLite, sin servidor.
  - **Ubicación:** los datos se guardan en `%APPDATA%\com.estra.bankomunales\`.
  - **Separación por banco:** `app.db` guarda el catálogo de Bankomunales, y cada banco tiene su propio archivo en `bankomunales\<id>.db`. Así, los datos de un banco nunca se mezclan con los de otro.
- **Contabilidad consistente:**
  - Cada operación (compra de acciones, desembolso, pago, cierre) registra automáticamente su asiento en el Libro de ingresos y egresos.
  - Las operaciones de varios pasos son atómicas: si algo falla a la mitad, se deshacen completas.
  - No se puede cerrar un mes si el balance no cuadra.
- **Respaldos:** son una copia consistente del archivo del banco, hecha con `VACUUM INTO`. Solo se pueden restaurar en el mismo Bankomunal del que salieron.

---

## Instalación (usuario final)

Requisitos: **Windows 10 u 11 de 64 bits**. No necesita internet.

1. Consiga el instalador `bankomunales_0.1.0_x64-setup.exe`. Para generarlo, vea [Generar el instalador](#generar-el-instalador).
2. Haga doble clic en el instalador y siga los pasos. Si Windows muestra *"Windows protegió su PC"*, elija **Más información → Ejecutar de todas formas**. Ese aviso aparece porque el programa no está firmado digitalmente.
3. Abra **Bankomunales** desde el menú Inicio.
4. Ingrese con la contraseña del sistema y cree su primer Bankomunal.

**Actualizar a una versión nueva:** instale el nuevo instalador encima del anterior. Los datos no se pierden, porque se guardan aparte en `%APPDATA%`. De todos modos, **haga un respaldo antes de actualizar**.

El uso detallado de cada pantalla está en el [Manual de Usuario](docs/Manual_de_Usuario_Bankomunales.docx).

---

## Instalación para desarrollo

### Requisitos

| Herramienta | Versión | Nota |
|---|---|---|
| [Node.js](https://nodejs.org/) | 20 o superior | |
| [Rust](https://rustup.rs/) | estable reciente | se instala con `rustup` |
| [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) | — | carga de trabajo *"Desarrollo para el escritorio con C++"* |
| WebView2 | — | ya viene en Windows 10/11 |

Más detalle en los [prerrequisitos de Tauri](https://tauri.app/start/prerequisites/).

### Pasos

```bash
git clone https://github.com/Sarasaenz12/bankomunales.git
```

```bash
cd bankomunales
```

```bash
npm install
```

```bash
npm run tauri dev
```

`npm run tauri dev` abre la aplicación con recarga en caliente. La primera compilación de Rust tarda varios minutos.

> **Datos de prueba sin tocar los reales:** si se define la variable de entorno `BANKOMUNALES_DATA_DIR` con otra carpeta, la aplicación guarda ahí sus datos en lugar de `%APPDATA%`.

### Pruebas y revisión

```bash
cd src-tauri && cargo test
```

```bash
cd src-tauri && cargo clippy --all-targets
```

```bash
npx tsc --noEmit
```

---

## Generar el instalador

```bash
npm run tauri build
```

Cuando termina, los instaladores quedan en:

| Archivo | Uso |
|---|---|
| `src-tauri/target/release/bundle/nsis/bankomunales_0.1.0_x64-setup.exe` | **Instalador recomendado** (asistente normal de Windows) |
| `src-tauri/target/release/bundle/msi/bankomunales_0.1.0_x64_en-US.msi` | Alternativa en formato MSI |

---

## Estructura del proyecto

```
bankomunales/
├── src/                      # Interfaz (React + TypeScript)
│   ├── app/                  # Rutas y layout principal
│   ├── core/                 # API hacia el núcleo, tipos, utilidades comunes
│   └── features/             # Una carpeta por pantalla/módulo
│       ├── auth/  dashboard/  configuracion/  socios/  acciones/
│       ├── creditos/  deuda/  caja/  cierre/  liquidacion/
│       └── reportes/  respaldo/  auditoria/
├── src-tauri/                # Núcleo (Rust)
│   ├── src/
│   │   ├── core/             # Base de datos, migraciones, errores, atomicidad
│   │   ├── modules/          # Un módulo por área de negocio
│   │   │   └── <módulo>/     # domain.rs · application.rs · data.rs · presentation.rs
│   │   ├── lib.rs            # Composición de dependencias y registro de comandos
│   │   └── state.rs          # Estado de la sesión
│   └── tauri.conf.json       # Configuración de la app y del instalador
├── docs/                     # Documentación del proyecto y manual de usuario
└── public/                   # Logo e imágenes estáticas
```

---

## Documentación

En la carpeta [`docs/`](docs/):

- **[Manual de Usuario](docs/Manual_de_Usuario_Bankomunales.docx):** guía paso a paso de cada pantalla, con capturas.
- **Documento de entendimiento:** requerimientos funcionales (RF) y reglas de negocio (RN).
- **Arquitectura y diseño técnico** y **diagramas técnicos**.
- **Diagramas** en draw.io: contexto, casos de uso, clases, componentes y arquitectura.
- **Decisiones pendientes:** temas que el cliente aún debe definir.
