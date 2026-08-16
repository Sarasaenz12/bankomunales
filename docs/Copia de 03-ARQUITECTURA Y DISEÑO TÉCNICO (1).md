# **ARQUITECTURA Y DISEÑO TÉCNICO** 

|**Nombre del proyecto:**|Bankomunales|
|---|---|
|**Cliente:**|Fundación Smurfit westrock|
|**Versión:**|1.0|
|**Creado por:**|Sara Valentina Sánchez Estrada|



## **1. Stack Tecnologico** 

|**Capa**|**Tecnologia seleccionada**|**Justificacion**|
|---|---|---|
|**Frontend**|React (TypeScript)|Se renderiza dentro del WebView2<br>nativo de Windows que usa Tauri|
|**Backend / API**|Rust, mediante Comandos<br>Tauri|Alto rendimiento y bajo consumo<br>de RAM frente a Node.js (Electron);<br>aquí vive la capa de<br>Dominio/Servicios con las Reglas<br>de Negocio|
|**Base de datos**|SQLite|Motor embebido, sin servidor, ideal<br>para app 100% offline; un<br>archivo .db independiente por cada<br>Bankomunal para garantizar el<br>aislamiento de datos|
|**Autenticacion**|Contraseña genérica<br>compartida, con hash<br>(Argon2/bcrypt)<br>almacenado localmente|No hay usuarios individuales ni<br>sesiones remotas es un candado de<br>acceso a la app, no un sistema de<br>identidad de usuarios.|
|**Almacenamiento**|Sistema de archivos local<br>(API de Rust / plugin)|Para los archivos de respaldo y los<br>reportes exportados; no requiere<br>almacenamiento en la nube.|
|**Infraestructura /**<br>**Cloud**|No aplica — despliegue<br>local (On-Premise)|La app corre 100% en el<br>computador del Bankomunal, sin<br>servidor ni nube, por requisito del<br>proyecto|
|**CI/CD**|GitHub Actions|Automatiza la generación del<br>instalador .exe/.msi con tauri build<br>en cada versión, sin implicar<br>despliegue remoto.|
|**Monitoreo / Logs**|Logging local en archivo|Al no haber servidor central, no hay<br>telemetría en la nube; los logs<br>quedan en el mismo computador<br>para depuración local si algo falla.|
|**Testing**|Cargo test (Rust, para las<br>Reglas de Negocio) +<br>Vitest/React Testing Library<br>(componentes React)|Permite probar la capa de Dominio<br>de forma aislada, independiente de<br>la interfaz coherente con la<br>separación en capas.|



## **2. Arquitectura General del Sistema** 

|**Patrón arquitectónico:**|Monolito Modular Multicapa 4 capas (Presentación,<br>Aplicación/Servicios, Dominio, Datos), organizadas<br>también por módulo de negocio (Socios, Acciones,<br>Créditos, etc.).|
|---|---|
|**Estilo de comunicación:**|IPC nativo de Tauri — comandos invoke()<br>(petición/respuesta, equivalente local a lo que sería un<br>REST) y eventos emit()/listen() (para notificaciones del<br>backend hacia el frontend, equivalente local a lo que<br>sería un WebSocket). No se usa HTTP ni sockets de red,<br>porque ambas capas corren en el mismo proceso.|
|**Tipo de despliegue:**|Local instalador de escritorio para Windows generado<br>con el empaquetador de Tauri (.exe/.msi), instalado de<br>forma independiente en cada computador o sede (Pijao,<br>La Tebaida).|
|**Entornos:**|Desarrollo (tauri dev, con recarga en caliente) y<br>Producción (build final empaquetado). No aplica un<br>entorno de "Staging" tradicional, ya que no hay servidor<br>remoto donde desplegar una versión intermedia — la<br>validación con el cliente se hace directamente sobre<br>builds de prueba instaladas en su propio computador.|



### **2.1 Descripción de Capas** 

|**Capa**|**Responsabilidad**|
|---|---|
|**Presentación / UI**|Interfaz en React; se comunica con el backend exclusivamente<br>mediante comandos Tauri (invoke()), no mediante una API HTTP.|
|**Logica de negocio**|Implementada en Rust: valida y aplica las Reglas de Negocio<br>(RN-01 a RN-15) y orquesta cada Caso de Uso|
|**Acceso a datos**|Capa en Rust que ejecuta las consultas SQL contra SQLite,<br>aislada de la lógica de negocio.|
|**Base de datos**|SQLite — un archivo .db por Bankomunal, persistencia local sin<br>servidor.|





<!-- Start of picture text -->
Aplicacion de escritorio<br>Tauri Shell = Ventana = WebView? «<br>Plugins<br>Frontend - React<br>Presentaci6n Componentes (Hooks,sipsi osContext,PineEstado)<br>Invaket} Respuesta<br>Backend - Rust<br>Comandos Tauri<br>Logica de datos<br>Repositorio<br>(acceso rapido)<br>S$Olite<br><!-- End of picture text -->

|**Módulo**|**Funcion**|**Depende de**|
|---|---|---|
|**Frontend - React**|Interfaz de usuario (formularios,<br>tablas, reportes en pantalla);<br>envía comandos al backend.|reportes en pantalla); envía<br>comandos al backend.Los 10<br>módulos del Backend - Rust (vía<br>invoke())|
|**Módulo**<br>**Autenticación**|Valida la contraseña genérica y<br>gestiona la creación/selección<br>del Bankomunal activo.|Persistencia SQLite|
|**Módulo**<br>**Configuración**|Gestiona los parámetros del<br>Bankomunal (tasas, % de<br>garantía, % de fondos, valor<br>nominal, PPCFC).|Persistencia SQLite|
|**Módulo Socios**|Registro, consulta y<br>actualización de socios,<br>beneficiarios y protegidos.|Persistencia SQLite|
|**Módulo Acciones**|Compra, liquidación y reparto de<br>ganancias de acciones.|Persistencia SQLite|
|**Módulo Créditos**|Solicitud, aprobación,<br>desembolso, pagos y<br>refinanciamiento de créditos.|Persistencia SQLite|
|**Módulo Caja y**<br>**Contabilidad**|Registro de movimientos del<br>Libro de Ingresos y Egresos.|Persistencia SQLite|
|**Módulo Cierre**<br>**Mensual**|Ejecuta el Proceso de Cuadre y<br>Cierre, calcula Activo/Pasivo.|Persistencia SQLite|
|**Módulo Reportes**|Genera los 9 reportes<br>administrativos y financieros.|Persistencia SQLite|
|**Módulo Respaldo**|Genera y restaura archivos de<br>respaldo del Bankomunal.|Persistencia SQLite, Sistema de<br>Archivos|
|**Persistencia**<br>**SQLite**|Almacena y recupera los datos<br>del Bankomunal (un archivo.db<br>independiente por Bankomunal).|<br>es la capa más baja, no<br>depende de otro componente|





<!-- Start of picture text -->
| Aplicacién de Escritorio (Tauri Shell) i<br>i pF FSET SSI EIDE ISIIIS Frontend- React (UI,Presentacion) a {<br>i Toda comunicacién Froiitend--Babkend ocurre via invoke() : IComandos Tauri (comandos #{taur|::command)), Sin red ni servidor HTTP de por medio. i i ' H<br>| | Backend-Rust  } ' f H H ' Ht i i |<br>H i Modulo Autenticacion Modulo Configuracion Modulo Socios Modulo Acciones Modulo Créditos q i<br>i i Médulo Caja y Contabilidad Médulo Cierre Mensual Modulo Reportes Modulo Respaldo Médulo Auditoria ' '<br>{ : CD — C4 — Cc 4 i<br>fi ii | eeeehesccscssgfscssscessccnccesscssslesfpecsesesseceessscsssscsssnsssssssape i<br>i ana ee i<br>' ' Neenececcccceeecceeeec sc eeeec ee peneeceee scene? Se e------larchive -------5, H H<br>Cc i C_] .<br>L_PersistenciaSQLitg (por Bankomunal) —Sistemade Archivos (Respaldo/Restauracion)<br><!-- End of picture text -->



<!-- Start of picture text -->
Configuracion<br>a<br>id:ooUUTextPA<br>(aor‘Shimoneda.pct_fondo_gastos nominalTondoTEXT gotsscion:-DECIMAL DECIMALDECIMAL TTwuaa<br>pel lone nbroblesECIMAL g Se wu<br>Selopeck-fongetfondoeobrablesOECIMAL mes.compra DATE<br>ref ranped-pet.pel DE CIMALCAL canta INTEGER<br>topes, menaua pt: DECIMAL<br>|___pave.oanancia<br>g Meitodo:feenapage:mentopatSeconwUD“DECALganasTEXTIPAUATE(enum)4: UID18 UUID IF IFS<br>he:Sostombee.tenetuux  exTEXT-Uul IF|' |_repertogenencienae<br>ceouis.parentescoTEXT :TEXT socio WUD‘ierre_mes_id PS -UID (FX)<br>id UID [PK mes : DATE<br>= veunen 4 Searorarcevre<br>pomsres Text fondoLoastos DECIMAL ome<br>bxoteson:Girecionspams TEXTTEXTTEXT sconesSoncecnete.‘iorachvasDECIMALINTEGER<br>Ceuta TEXt por action DEGRA<br>SoresSts TEXTTEXT (num)<br>protege- fecha_ingresofecnaconeSSsecresble” OATE: DATEDECIMAL _vuSinesmontefasnplazo_cuotasSteSaldo‘pendinteDECMALseualegal:TEXTee)PK : INTEGERNTEGERDECIMALDECIMAL__|<br>WUaosombre.teouls. TEXTTEXT b {SonnssiousfechoSesemboso.¥nonctaTEXT (ram)‘DATEDATE<br>forenesco TEXT<br>Caja, Cierre y Auditoria ae<br>imovininta_ bro | ‘id:BeatotaUUID [PK]vu ri |<br>fa:eetrestleromeroUDAf UUDDINTEGERPK)UUIDIF. rkopcionalpean, fo.Seeones_conprometdes TEXT(TTTULARFIADOR)DECIMAL<br>fone ATE<br>Scgoescripcion:woreseSpeso TEXTDESMASeCnNALTEXT |__siememes— Rospalaopal id:Motnumero.WUD PK)INTEGERUUIO IK<br>sso" oecmMaL id WUD PA —> tech. vencnens DATE<br>‘efectivoMes ORTE:DECIMAL a‘id: UUID (PK) feonactecapital :DECIMALpage, bale<br>waudit o ria carteraSonesteepasivo_totalfat:DESIMALDECIMALDECIMAL-DECIMAL fechatipowoven.tthe - TEXT :DATETIMETEXT (RESPALDO/RESTAURACION)TEXT Vatordias_atrasointeresposude lola: DECIMAL BOOLEAN :DECIMAL INTEGER<br>id:igromeretidodstectogeUUIDWUDPAeen(PK) rekza:TEXTTEXT| | {SennEcce oenesooteanDATETIME<br>‘aioompo-imeciicadovaocueesstonerTEXTTextTEXT<br>moto TEXT<br>Morocco TEXT (enum)<br><!-- End of picture text -->



|saldo_fondo_incobrables|DECIMAL(14,2)|Si|
|---|---|---|
|ppcfc_rango1_pct|DECIMAL(5,2)|Si|
|ppcfc_rango2_pct|DECIMAL(5,2)|Si|
|tope_individual_mensual_pct|DECIMAL(5,2)|Si|





#### **Tabla: socio** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|cedula|VARCHAR(20)|Si||
|nombres|VARCHAR(100)|Si||
|apellidos|VARCHAR(100)|Si||
|profesion|VARCHAR(100)|Si||
|direccion|VARCHAR(200)|Si||
|telefono|VARCHAR(20)|Si||
|celular|VARCHAR(20)|Si||
|correo|VARCHAR(100)|Si||
|estatus|VARCHAR(30)|Si||
|fecha_ingreso|DATE|Si||
|fecha_retiro|DATE|No||
|saldo_incobrable|DECIMAL(14,2)|No||



#### **Tabla: beneficiario** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|socio_id|UUID|Si|FK<br>socio<br>→|
|nombre|VARCHAR(100)|Si||
|cedula|VARCHAR(20)|Si||



|parentesco|VARCHAR(50)|Si|
|---|---|---|



#### **Tabla: protegido** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|socio_id|UUID|Si|FK<br>socio<br>→|
|nombre|VARCHAR(100)|Si||
|cedula|VARCHAR(20)|Si||
|parentesco|VARCHAR(50)|Si||



#### **Tabla: lote_acciones** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|socio_id|UUID|Si|FK<br>socio<br>→|
|mes_compra|DATE|Si||
|cantidad|INTEGER|Si||
|liquidada|BOOLEAN|Si||



#### **Tabla: reparto_ganancias** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|cierre_mes_id|UUID|Si||
|mes|DATE|Si||
|total_ingresos_repartibles|DECIMAL(14,2)|Si||
|fondo_incobrables|DECIMAL(14,2)|Si||



|fondo_gastos|DECIMAL(14,2)|Si|
|---|---|---|
|balance_neto|DECIMAL(14,2)|Si|
|acciones_activas|INTEGER|Si|
|valor_por_accion|DECIMAL(12,2)|Si|





#### **Tabla: pago_ganancia** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Si|PK|
|lote_acciones_id|UUID|Si|FK<br>→<br>lote_acciones|
|reparto_ganancias_id|UUID|Si|FK<br>→<br>reparto_ganan<br>cias|
|monto|DECIMAL(12,2)|Si||
|estado|VARCHAR(20)|Si||
|fecha_pago|DATE|Si||



#### **Tabla: credito** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|numero|VARCHAR(20)|Sí|UNIQUE|
|monto_original|DECIMAL(14,2)|Sí||
|tasa|DECIMAL(5,2)|Sí||
|plazo_cuotas|INTEGER|Sí||
|cuota_actual|INTEGER|Sí||
|saldo_pendiente|DECIMAL(14,2)|Sí||



|destino|VARCHAR(10)|Sí|
|---|---|---|
|estatus|VARCHAR(20)|Sí|
|fecha_solicitud|DATE|Sí|
|fecha_desembolso|DATE|Sí|





#### **Tabla: garantia_credito** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|credito_id|UUID|Sí|FK<br>credito<br>→|
|socio_id|UUID|Sí|FK<br>socio<br>→|
|rol|VARCHAR(10)|Sí||
|acciones_comprometidas|DECIMAL(12,2)|Sí||



#### **Tabla: cuota** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|credito_id|UUID|Sí|FK<br>credito<br>→|
|numero|INTEGER|Sí||
|fecha_vencimiento|DATE|Sí||
|fecha_pago|DATE|No||
|capital|DECIMAL(12,2)|Sí||
|interes|DECIMAL(12,2)|Sí||
|valor_total|DECIMAL(12,2)|Sí||
|pagada|BOOLEAN|Sí||
|dias_atraso|INTEGER|Sí||



#### **Tabla: movimiento_libro** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|socio_id|UUID|No|FK<br>socio<br>→<br>(opcional)|
|credito_id|UUID|No|FK<br>credito<br>→<br>(opcional)|
|numero|INTEGER|Sí||
|fecha|DATE|Sí||
|codigo|VARCHAR(10)|Sí||
|descripcion|VARCHAR(200)|Sí||
|ingreso|DECIMAL(14,2)|Sí||
|egreso|DECIMAL(14,2)|Sí||
|saldo|DECIMAL(14,2)|Sí||



#### **Tabla: cierre_mes** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|mes|DATE|Sí||
|efectivo|DECIMAL(14,2)|Sí||
|cartera|DECIMAL(14,2)|Sí||
|bienes|DECIMAL(14,2)|Sí||
|activo_total|DECIMAL(14,2)|Sí||
|pasivo_total|DECIMAL(14,2)|Sí||
|cuadra|BOOLEAN|Sí||



fecha_cierre 

DATETIME Sí 

#### **Tabla: respaldo** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|fecha|DATETIME|Sí||
|usuario|VARCHAR(100)|Sí||
|ruta_archivo|VARCHAR(255)|Sí||
|tipo|VARCHAR(20)|Sí||



#### **Tabla: auditoria** 

|**Campo**|**Tipo**|**Obligatorio**|**Clave**|
|---|---|---|---|
|id|UUID|Sí|PK|
|fecha|DATETIME|Sí||
|nombre_quien_realiza|VARCHAR(100)|Sí||
|entidad_afectada|VARCHAR(50)|Sí||
|campo_modificado|VARCHAR(50)|No||
|valor_anterior|TEXT|No||
|valor_nuevo|TEXT|No||
|motivo|TEXT|No||
|tipo_accion|VARCHAR(30)|Sí||



## **5. Especificación de APIs** 

Documenta los endpoints principales del sistema. Para proyectos grandes, considera usar Swagger/OpenAPI. 

|**Método**|**Endpoint**|**Descripción**|**Body / Params**|**Respuesta**|
|---|---|---|---|---|
|**POST**|/api/auth/register|Registrar nuevo<br>usuario|{ email,<br>password, name<br>}|201: { user,<br>token }|
|**POST**|/api/auth/login|Iniciar sesión|{ email,<br>password }|200: { token }|
|**GET**|/api/users/:id|Obtener usuario por<br>ID|Header:<br>Authorization|200: { user }|
|**PUT**|/api/users/:id|Actualizar usuario|{ name, email }|200: { user }|
|**DELETE**|/api/users/:id|Eliminar usuario|Header:<br>Authorization|204: No content|





## **6. Registros de Decisiones de Arquitectura (ADR)** 

Los ADR documentan el razonamiento detrás de cada decisión técnica importante. Permiten que futuros ingenieros entiendan el PORQUÉ del diseño. 

|**ADR-01**<br>**Arquitectura de software: Monolito Modular Multicapa**|
|---|



|**Estado:**|Aceptado|
|---|---|
|**Contexto:**|Bankomunales es una app de escritorio 100% local/offline, sin<br>servidor. Debe organizar 10 módulos de negocio, cada uno<br>con sus propias Reglas de Negocio, sin mezclar lógica de<br>negocio con interfaz ni con acceso a datos.|
|**Decisión tomada:**|Arquitectura monolítica (un solo proceso, sin red), organizada<br>en 4 capas — Presentación, Aplicación/Servicios, Dominio,<br>Datos — y a la vez dividida por módulo de negocio (Socios,<br>Acciones, Créditos, etc.), cada uno con sus propias 4 capas<br>internas.|
|**Consecuencias:**|Si se cambia el framework de UI en el futuro, solo se<br>reescribe la capa de Presentación. Las Reglas de Negocio<br>quedan centralizadas y testeables por separado. Requiere<br>disciplina para no mezclar lógica de negocio dentro de la UI<br>bajo presión de tiempo.|
|**Alternativas**<br>**consideradas:**|Microservicios (descartada, no hay red que lo justifique) —<br>Monolito sin capas (descartada, mezclaría reglas de negocio<br>con UI).|



|**ADR-02**|**Stack tecnológico: Tauri (React + Rust) sobre Electron y**<br>**Flutter**|
|---|---|
|**Estado:**|Aceptado|
|**Contexto:**|La app debe correr en computadores de gama modesta,<br>instalarse fácil en Windows, y el equipo de desarrollo tiene<br>experiencia previa en React, sin experiencia en Dart ni Rust.|
|**Decisión tomada:**|Tauri, con Frontend en React (reutiliza el conocimiento<br>previo) y Backend/Lógica en Rust (bajo consumo de RAM,<br>~30-80MB vs. ~150-300MB de Electron).|
|**Consecuencias:**|Se resuelve el problema de rendimiento en hardware<br>modesto sin perder el conocimiento previo en React. Existe<br>una curva de aprendizaje nueva en Rust, mitigada porque la<br>lógica de negocio (validaciones porcentuales, sumas, tablas<br>de amortización) es aritmética simple.|
|**Alternativas**<br>**consideradas:**|Electron (descartada por RAM/tamaño de instalador) —<br>Flutter Desktop (descartada, obligaba a aprender Dart para<br>toda la app, riesgo mayor de tiempo en una práctica<br>universitaria).|



|**ADR-03**|**Base de datos: SQLite con un archivo independiente por**<br>**Bankomunal**|
|---|---|
|**Estado:**|Aceptado|
|**Contexto:**|La app debe funcionar offline y permitir que varios<br>Bankomunales (Pijao, La Tebaida) coexistan en el mismo<br>computador o en instalaciones separadas, sin mezclar sus<br>datos.|
|**Decisión tomada:**|SQLite, con un archivo .db independiente por cada<br>Bankomunal. Ninguna tabla lleva una columna<br>bankomunal_id, porque el aislamiento se da por archivo, no<br>por filtro de datos.|
|**Consecuencias:**|Aislamiento de datos garantizado a nivel de archivo — es<br>físicamente imposible mezclar datos entre Bankomunales,<br>incluso si hay un error de programación en una consulta. El<br>Respaldo se simplifica a copiar el archivo completo. Si en el<br>futuro se quiere consolidar reportes de varios Bankomunales<br>en un solo lugar, se necesitaría un proceso de importación<br>aparte (no soportado nativamente).|
|**Alternativas**<br>**consideradas:**|Una sola base de datos con columna bankomunal_id en cada<br>tabla (descartada, mayor riesgo de mezcla accidental de<br>datos) — Motor con servidor tipo PostgreSQL (descartada,<br>requiere proceso de servidor, incompatible con app 100%<br>local).|



|**ADR-04**<br>|**Autenticación sin control de acceso por roles,**<br>**compensado con Auditoría**<br>|
|---|---|
|**Estado:**|Aceptado|
|**Contexto:**|El BkSistema anterior no diferenciaba usuarios individuales;<br>los 4 roles de la Junta (Verificador, Cajero, Contable,<br>Actualizador) son rotativos entre los mismos socios, y el<br>cliente confirmó explícitamente que el sistema no debe<br>bloquear funciones por rol.|
|**Decisión tomada:**|Login con una única contraseña genérica compartida (sin<br>usuarios ni roles con permisos distintos). Como<br>compensación, toda acción sensible (cambios de<br>configuración, correcciones tras cierre de mes,<br>respaldo/restauración) exige que la persona escriba su<br>nombre, y esto queda registrado en una bitácora de Auditoría|



||— solo para trazabilidad, no como control de acceso.|
|---|---|
|**Consecuencias:**|El sistema es simple de usar para una comunidad sin perfil<br>técnico. No hay forma de impedir que alguien haga algo fuera<br>de su rol asignado en el papel — pero sí queda registro de<br>quién lo hizo, para revisar después si hace falta.|
|**Alternativas**<br>**consideradas:**|Login individual por usuario con permisos por rol (descartada,<br>contradice cómo opera realmente la Junta rotativa) —<br>PIN/firma digital por operación sensible (descartada,<br>reintroduce fricción de control de acceso que el cliente ya<br>rechazó).|



## **7. Consideraciones de Seguridad** 

|**Área**|**Medida implementada**|
|---|---|
|**Autenticación**|Contraseña genérica compartida, sin sesiones ni tokens (no<br>hay red entre Frontend y Backend que requiera JWT).|
|**Autorización**|No hay control por roles (RBAC) — decisión explícita del<br>cliente (ver ADR-04). Se compensa con la bitácora de<br>Auditoría, no con permisos.|
|**Contraseñas**|Hash con Argon2 o bcrypt (mínimo 10 rondas), nunca<br>almacenada en texto plano.|
|**Comunicación**|No aplica HTTPS — Frontend y Backend corren en el mismo<br>proceso; se comunican porinvoke()(IPC interno de Tauri),<br>sin red ni puertos abiertos de por medio.|
|**Inyeccion SQL**|usar siempre consultas parametrizadas (rusqlite/sqlx),<br>nunca concatenar texto del usuario directamente en el SQL.|
|**Datos sensibles**|El archivo .db de SQLite no está cifrado por defecto, y<br>contiene cédulas e información financiera de los socios — se<br>recomienda evaluar cifrado en reposo (ej. SQLCipher) si el<br>computador no tiene otra protección (BitLocker, contraseña<br>de Windows).|
|**Ataques por red/IP**|Tauri no levanta ningún servidor HTTP ni abre puertos de red<br>para su funcionamiento normal; toda la comunicación ocurre<br>dentro del mismo proceso del sistema operativo. No hay una<br>IP a la que un atacante externo pueda apuntar.|



|**Seguridad física**|robo o pérdida del computador, o de un USB con un respaldo.<br>Se recomienda cifrar también los archivos de respaldo que<br>salgan del equipo.|
|---|---|
|**Integridad del**<br>**instalador**|Firmar digitalmente el instalador (.exe/.msi) generado por<br>Tauri, para que Windows no lo marque como "editor<br>desconocido" y para evitar que alguien distribuya una versión<br>alterada.|



