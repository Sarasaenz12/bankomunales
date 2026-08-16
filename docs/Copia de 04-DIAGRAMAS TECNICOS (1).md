# **DIAGRAMAS TECNICOS** 

|**Nombre del proyecto:**|Bankomunales|
|---|---|
|**Cliente:**|Fundación Smurfit westrock|
|**Versión:**|1.0|
|**Creado por:**|Sara Valentina Sánchez Estrada|



### **Orden recomendado de elaboración** 

|**#**|**Diagrama**|**Fase donde se elabora**|**Cuándo se realiza**|
|---|---|---|---|
|1|Casos de uso|_Inicio — Análisis_|Después del<br>levantamiento de<br>requisitos|
|2|Diagrama de contexto|_Inicio — Análisis_|Antes de hablar de<br>código — 1ra reunión|
|3|Arquitectura del sistema|_Planificación — Diseño_|Antes de escribir<br>código|
|4|Entidad-Relación (ER)|_Planificación — Diseño_|Al diseñar la base de<br>datos|
|5|Diagrama de flujo|_Planificación — Flujos_|Al definir los<br>procesos principales|
|6|Diagrama de secuencia|_Planificación — Flujos_|Al definir flujos entre<br>módulos/APIs|
|7|Diagrama de despliegue|_Infraestructura_|Antes de configurar<br>servidores|
|8|Diagrama de<br>componentes|_Infraestructura_|Proyectos con<br>múltiples módulos|
|9|Diagrama de clases<br>(opcional)|_Si se usa OOP_|Si el proyecto usa<br>OOP / patrones|
|10|Wireframes / Mockups<br>(opcional)|_Si hay interfaz de usuario_|Antes de desarrollar<br>la UI|





<!-- Start of picture text -->
yasa ae<br>i <fites cen) VW<br>po =—12}<br>Sh<br>Ss os<br><!-- End of picture text -->



<!-- Start of picture text -->
/~——utencaciony Usuarios Conguracion del Bankomunal Gestionde Accones<br>LS<br>\= 0 ae aee =<br>Verficad crear Bankomunal cn Liquidar Acciones 74 |<br>( Gestion de Socios ) | || tualzador<br>‘SeleccionarActivo Bankomunal ~~>—-+ || eee Repartir Ganancias \ I<br>‘Consultar/Actualizar Datos de’<br>| {ULL _<br>© yfyf ft |<br>\ eH o<br>A se|Caja y Contabiidad<br>" —egistrar Otros<br>Cajero i Reaistrar Oto |<br>rit ‘AprobariNegar/Diferir Ingresos/Egresos |<br>en<br>a.[|| —-ee<br>rire |= o>a Gestonar“Fond de<br>HL Cuota |<br>AAI Registrar Pago de Deuda Corregir Una Operacion<br>ree<br>SeeeA<br>tC Ty. |<br>| | ST<br>—— [> SSS No<br>= IS Generar Reportes _—) Respalar Informacion<br>= = 7<br><!-- End of picture text -->



<!-- Start of picture text -->
Socio del banka<br>Administrador del Verificador<br>sistema<br>RK ‘Compra, Liquidacién de<br>Configuracion Bankomunal, accciones, Solicita creditos Valida opreaciones,“a<br>spe Aprueba movimientos<br>Sistemas<br>DifiereAprueba/Niega/Solicitudes Ban kom unales RegistraDesembolsos,pagos,<br>Junta Administradora de credito Ingresos/egresos Cajero<br>. . Caudre de cierre,<br>Registra Socios, Fondo de gastos,<br>Actualiza Datos Correcciones<br>Actualizador Contable<br><!-- End of picture text -->

**Stack tecnológico del proyecto:** 

|**Capa**|**Tecnologia seleccionada**|**Justificacion**|
|---|---|---|
|**Frontend**|React (TypeScript)|Se renderiza dentro del<br>WebView2 nativo de<br>Windows que usa Tauri|
|**Backend / API**|Rust, mediante Comandos<br>Tauri|Alto rendimiento y bajo<br>consumo de RAM frente a<br>Node.js (Electron); aquí vive<br>la capa de Dominio/Servicios<br>con las Reglas de Negocio|
|**Base de datos**|SQLite|Motor embebido, sin<br>servidor, ideal para app<br>100% offline; un archivo .db<br>independiente por cada<br>Bankomunal para garantizar<br>el aislamiento de datos|
|**Autenticacion**|Contraseña genérica<br>compartida, con hash<br>(Argon2/bcrypt) almacenado<br>localmente|No hay usuarios individuales<br>ni sesiones remotas es un<br>candado de acceso a la app,<br>no un sistema de identidad<br>de usuarios.|
|**Almacenamiento**|Sistema de archivos local<br>(API de Rust / plugin)|Para los archivos de<br>respaldo y los reportes<br>exportados; no requiere<br>almacenamiento en la nube.|
|**Infraestructura / Cloud**|No aplica — despliegue local<br>(On-Premise)|La app corre 100% en el<br>computador del Bankomunal,<br>sin servidor ni nube, por<br>requisito del proyecto|
|**CI/CD**|GitHub Actions|Automatiza la generación<br>del instalador .exe/.msi con<br>tauri build en cada versión,<br>sin implicar despliegue<br>remoto.|
|**Monitoreo / Logs**|Logging local en archivo|Al no haber servidor central,<br>no hay telemetría en la nube;<br>los logs quedan en el mismo<br>computador para depuración<br>local si algo falla.|



_Versión   V1_ 



<!-- Start of picture text -->
Aplicacion de escritorio<br>Tauri Shell « Ventana = WebView2 «<br>Plugins<br>Frontend - React<br>=n Logica de la aplicacion<br>Presentacion Componentes (Hooks, Context, Estado)<br>Invoake() Respuesta<br>Backend - Rust<br>Comandos Tauri<br>Logica de datos<br>Repositorio<br>(acceso rapido)<br>SOQlite<br><!-- End of picture text -->



<!-- Start of picture text -->
Configuracién<br>bankomunal<br>ia:tomeUUD TEXTPK)<br>ValomonedanominalTEXTaccion: DECIMAL lote_accones<br>pel fondo gastos“DECMAL _—<br>Sek. ondo gatos DECIMAL WUD<br>pct_fondo_incobrablesSal.Ppcfc_rango1_pctfondocobbles-DECIMAL:DECIMALDECIMAL < mmes_compracantidadnS «i DATEDATE<br>top.ppcfc_rango2_pctnaradul menaual-DECIMALpot:DECIMAL Gudtete: GOOLEAN<br>|___Peve.genancia<br>ig-UUD Pd<br>q Ifefecha_pago:tepatogananiasimentestado:-accons_id:“BECIMALTEXT (enum)DATEUUid: UID(Fi<br>benefciaro<br>ig: vUD PK)<br>sociocedula.parentecofond¢ UDF)TEXTurnsTEXT | |___oesceaulaapelidoswuteeTEX!PeTEXTsaxoumtaue)| < fondoiaSerres{ofal_ingresos_repartbiesfendo-gastosmesUUDATE mes.ncotrablesreparto_ganancianPK DECIMALUUDDECIMAL(Fe  :DECIMAL<br>|__proteidoidSecond:nombrecedua.parentescoUD :TEXTTEXT(PKuur:;TEXT ||r brctesiongressionceblaTEXTcomeostats:fecha_ingresofecha-retoSalde_ncobrableproteson: :TEXTTEXTTOTETEXTDATE :enum)DATE“DECIMAL VBlorbalance-neto-accones,scvasporecoonDECIMALINTEGERDECIMAL La:fecha_solicitudigmonotase’plazo_cuotasStota-actual-Saido_pendentedesingestostecha-desemboto uoeDECIMALnginalTEXTTEXTeroditopa: : INTEGER:NTEGER(enum)(enum):DECIMALDATEDECIMALDATE<br>Caja, CierreiaSocioUUDaa)id  yUUIDPKAuditoria[FX opcional / 4 id:credsocto. i UIDTEXTto_ido.  CTULARIFADOR)=U:[PK]U U ID(rs5(Fx L<br>credito.idmumero INTEGER :UID [FK. opesonal ‘acciones_comprometidas :DECIMAL<br>teeniecodigodevcrpcon: :DATETEXTTEXT Pr, Respaldo quercuota<br>bebnedingresosaido’ DECIMAL=DECIMAL |-————i‘efectivofarlremes UDP : DATE.ciene_ :DECIMALDECIMALmee -uupea‘idfecha- UUIDDATETIME(PK) respaldo Sedonumero:fechafecha_de_pagocapitalcapital:vencmiento:DECIMALDECALINTEGERUD : DateIFDATE Pp<br>antteda dcivobienes tlaDECIMALDECIMAL vsvato:ta archivoTEXT TEXT RancevalortotalDEIMA<br>Laswuid:1gomtve.tntdad-stectadscampo UUD UUDjoditeado eaPAion (PK eaza:TEXT‘TEXTTEXT pesio.tundrafecha. ceretotal:BOOL-DDATETIME E ANCIMAL to “TEXT (RESPALDOIESTAURACION) bpagaa.  BOOLEAN<br>Valor attror: TEXT<br>mmotvetov acconTEXTTEXT (enum)<br><!-- End of picture text -->



<!-- Start of picture text -->
Ingresar Contrasefia<br>No<br><La contrasefia es<br>correcta?<br>SiL<br>v<br>y<br>N ~Existe un bankomunal<br>s creado?<br>—_<br>Presionar en crear<br>banco<br>Digitar nombre<br>del baco<br>Si<br>rN<br>gHay mas de un<br>SS Bankomunal?<br>Mostrar lista de<br>bancos<br>Seleccionar<br>bankomunales<br>Entrar al menu<br>principal<br><!-- End of picture text -->



<!-- Start of picture text -->
Registro de Socio<br>(E>» AcCeaa\ - _ ; Guardar socio<br>ome | Capturar datos personales —>No: Registrar beneficiario Registrar protegidos acai<br>$$ i es ————————EE<br>Rechazar.<br>cédula<br>duplicada<br><!-- End of picture text -->



<!-- Start of picture text -->
Ligucacionge Acctones(Parcialy Tota)<br>(ee<br>meee)ee |ee eeeTT) ‘om coee<br>(sree 7 Tamer mr 4 » paren<br>—<br>perrossem oor SSeS— Seteyomenany)ctgaaaman, etnco “ernpnse)saeute<br>femannend — (Severeees | (emesemnn<br>Wat "Sten tein” -* Sennen<br>nema wots _, (sews) _ 5estaund‘Omarsiua)omaRand sassa8 oron<br><!-- End of picture text -->



<!-- Start of picture text -->
Compra de Acciones<br>| comprade<br>5 Verificar tope 15% ¢Supera)<br>5 por socio tope? No<br>= oe<br>$ si<br>Rechazar!<br>solicitud<br>a »<br>Recibir pago<br>(cantidadx valor nominal)<br>—<*~——=z~——<br>= la »<br>2 Registrar ingreso en<br>5 caja y contabilidad<br>°<br>X J<br>:.Ss ;<br>ant Actualizar acciones de!<br>i]a=]<socioen el sistema )<br><!-- End of picture text -->



<!-- Start of picture text -->
a<br>| Gees) Sy<br>——_ ceeeneeientemtien | (™) (mm)<br>vale “cw = aes Lf Sectors) oye J)<br>|e| Som — esiarenet?| Sree arene —*("*)<br>i t LC =") Sea<br><!-- End of picture text -->



<!-- Start of picture text -->
|<br>E><br>von ©iSf ©)=) —— a oo=* omen _S<br>Commas= Sa aSee & Ss:<br>| 7 = | ee) (aT<br>Hover"soao ce > Conacne ot oe tonesoprictawGeraotonmascaeeemoreyreperes_, sore — papnsnanoneoguoresspoly |_| Ourencesinctea<br>tes<br>Sateen Sees cronse<br>Pa LF<br><!-- End of picture text -->



<!-- Start of picture text -->
Pagode Cuota<br>Ungareaniénala9<br>Oe —eseeeesesesai<br>Busca Eisstemaaia cuca<br>‘eco, interes despues‘2 dela fecha<br>| Sel cbt detsocio y Ersst e macalla  venoment>?RReciveaessocol nero de ee__—fecbeoperecn,yRegstaresumen elp390deat entege Socio.martes potene! e | _<br>2cam”ngresos |yp,<br>i SimeaaesPooch epee ec  — SSSit crbato e ne aor,S —. Sepvet<br><!-- End of picture text -->



<!-- Start of picture text -->
Pago de deuda pendiente<br>s >><br>a | pagar total o }<br>S|3 \parciaimente/Ey, (Fm)<br>r ~<br>Buscael listadoel crédito de créditosdel socio conen socioéAparece con .deuda: él. -Recibe el monto que. oeRegistra el pago en el zConfoN este pago a,El saldose actualiza pendiente EsAG?<br>SYdeuda en Incobrables. pendiente7 el~  sociopagar. va a abonar 0- a‘créditosistema, del socio asociadoretirado. al quedotodaXN la cancelada deuda?7 ———Actualiza el estatus del socio<br>7 de "Retirado con Deuda” a Fin (deuda<br>“Retirado con Deuda saldada).<br>Pagada"<br><!-- End of picture text -->



<!-- Start of picture text -->
|<br>‘wtranrsett su<br>O o ass se<br>‘Sowaciona G05108) (Catan mnoenen oe rors<br>Schnemfofrancarolssema |_,gejraiey—* cea,do mora peebendartes v0aI ‘tenoraes) | Permto oust tas 0 San‘ecocenta )<br>‘rdnanosbandana:0 more resumen‘racede mentosya Seco. erie<br>a —“—‘“irSi<br>gutsa snes epracenses| Fama obo y seo frm (rte)a)<br>, ™<br><!-- End of picture text -->



<!-- Start of picture text -->
Caja y Contabilidad<br>no relacionado a<br>~ Quéatipo de “s—=s—"") acciones ni créditos<br>operacion es? —<br>~~ Se registrara<br>como Ingreso al Registra fecha y - in<br>. automaticamente —____.<br>g Fondo para Gastos monto de la Si operactn pasa el registro al<br>g ——— oo operacion para Gastos Contable.<br>(ej. fotocopias, _correspondiente<br>transporte), afecta el _<br>Fondo para Gastos.<br>(ej. equipo, mobiliario, 0 algo<br>= comoen comodato), Activo Fijosey nocontabilizaafecta el<br>saldo de caja<br>2 Registra la operacion en el Libro de<br>= Ingresos y Egresos, con su numero<br>8 de operacion consecutivo.<br><!-- End of picture text -->



<!-- Start of picture text -->
\<br>GEI mes anterior .<br>sigue abierto?/<br>No vAJ<br>Ejecuta el Proceso de<br>Cuadre (puede repetirse<br>las veces que sea<br>necesario, es provisional).<br>| Elsistema genera el<br>Informe de Gestion<br>Mensual y el Balance del<br>Mes<br>> —No>, efrorUbica eny elcorrigelibro elde<br>, Ingresos y Egresos<br>Si<br>Ejecuta el cierre<br>mensual.<br>El sistema actualiza<br>Fondos de Gastos e<br>Incobrables segun %<br>| el sistema recalcula el<br>PPCFC incluyendo el<br>| mes recién cerrado.<br>el sistema sella el mes<br>como cerrado.<br><!-- End of picture text -->



<!-- Start of picture text -->
( Todos )<br>Entra al modulo de<br>Reportes.<br>| Selecciona el reporte |<br>deseado, de los 9<br>disponibles<br>(El sistema verifica si|<br>ese reporte necesita<br>parametros<br>\_adicionales.<br>El reporte _<br>de fechas? y<br>|<br>No<br>El sistema genera el<br>reporte con la<br>informacion solicitada.<br>iDesea El sistema exporta o<br>exportarlao =|——s envia a imprimir el<br>imprimirlo? reporte<br><!-- End of picture text -->

## **6. Diagrama de secuencia** 

Muestra el orden en que los componentes se comunican para ejecutar un proceso 

|**¿Qué es?**|Traza la interacción entre objetos, módulos o servicios en<br>orden cronológico.|
|---|---|
|**¿Cuándo hacerlo?**|Al definir flujos con múltiples servicios o APIs. Muy útil para<br>integrations.|
|**¿Quién lo revisa?**|Desarrolladores backend y equipos de integración.|
|**Duración estimada**|1–2 horas por flujo|



#### **Elementos que debe tener:** 

☐ _Participantes (actores o sistemas) en la parte superior — líneas de vida verticales_ ☐ _Mensajes horizontales entre participantes con flechas y etiquetas_ ☐ _Mensajes de retorno (respuesta) con líneas punteadas_ ☐ _Marcos de interacción: loop, alt, opt si hay condicionales o ciclos_ ☐ _Orden cronológico de arriba hacia abajo_ 

_Flujo representado:   ______________________________     Versión   _______ 

_[ Diagrama de Secuencia — pegar imagen o dibujar aquí ]_ 

## **7. Diagrama de Despliegue** 

Representa la infraestructura física y de red donde se ejecuta el sistema 

|**¿Qué es?**|Muestra los servidores, contenedores, nodos de red y cómo se<br>distribuye el software en ellos.|
|---|---|
|**¿Cuándo hacerlo?**|Antes de configurar servidores o contratar infraestructura<br>cloud.|
|**¿Quién lo revisa?**|DevOps, SysAdmin y desarrolladores backend.|
|**Duración estimada**|1–2 horas|



#### **Elementos que debe tener:** 

|☐|Nodos de despliegue: servidores físicos, VMs, contenedores Docker, funciones<br>serverless|
|---|---|
|☐|Artefactos desplegados en cada nodo (aplicaciones, APIs, bases de datos)|
|☐|Redes y zonas de seguridad (zona pública, privada, DMZ)|
|☐|Puertos y protocolos de comunicación entre nodos|
|☐|Servicios cloud si aplica (AWS, GCP, Azure) con sus nombres de servicio|



#### **Infraestructura del proyecto:** 

**Servidor / Servicio Proveedor Ambiente Responsable** _Ambiente:  ☐ Desarrollo   ☐ Staging   ☐ Producción Versión   _______ 

_[ Diagrama de Despliegue — pegar imagen o dibujar aquí ]_ 



<!-- Start of picture text -->
| Aplicacién de Escritorio (Tauri Shell) i<br>i pF FSET SSI EIDE ISIIIS Frontend- React (UI,Presentacion) a {<br>i Toda comunicacién Froiitend--Babkend ocurre via invoke() : IComandos Tauri (comandos #{taur|::command)), Sin red ni servidor HTTP de por medio. i i ' H<br>H | Backend-Rust } ' i H ' H tot i} ; H<br>H i Modulo Autenticacion Modulo Configuracion Modulo Socios Modulo Acciones Modulo Créditos q i<br>i i Médulo Caja y Contabilidad Médulo Cierre Mensual Modulo Reportes Modulo Respaldo Médulo Auditoria ' '<br>' : CD — C4 — Cc : H<br>fi ii | eeeehesccscssgfscssscessccnccesscssslesfpecsesesseceessscsssscsssnsssssssape i<br>i ana ee i<br>H ' Neenececcccceeecceeeec sc eeeec ee peneeceee scene? Se e------larchive -------5, H H<br>Cc i C_] .<br>L_PersistenciaSQLitg (por Bankomunal) —Sistemade Archivos (Respaldo/Restauracion)<br><!-- End of picture text -->



<!-- Start of picture text -->
Configuracion<br>Bankomunel<br>PIT)<br>Tonbre Stig<br>“Tannomalaeion:“monede: tteg Decimal<br>“purondoGestsFaPondoGactosDecimalDecal<br>“Puroncoincorales Decal<br>Fatarondoineotrables. Decimal<br>GoscRangetBer Decimal<br>FoccRongozeet Desial<br>oamnswatantenseaet Decal<br>+calcularPPCFC(): Decimal<br>Soplensvatsutreadod. Decimal .<br>‘<br>j a<br>Socios / Ih \ credios<br>Caja y Contabilidad<br>|___EstatssocioACTIVO,—- || a StingSocio “numer:MovimientoLibroneger “res DateCieretes [DestinoCredtio.‘anaaratons a Creito<br>RETIRADO_VOLUNTARIORETIRADO_CON_DEUDA -nombres:Spence: Stringsting Be Seago.fecha:  StegDate -efectivo:“Sater: DecimalDecimal AHARE -montoOnginal:‘Teen Deeral Decimal<br>RETIRADO_DEUOA SALDADA ly, | ayeccon,otesentelefono: String S tingtep ‘ere-egreso:“Gesetpcon DecalDecimalSing “SerenSotout-pasivoTotal: DecalCecilDecimal ew£D v n Fete “PasoBiman-saldoPendiente- c vcteuet megevDecimal e rr<br>celular: String : -saldo: Decimal -cuadra: Boolean GP crrrs=ess---0] destino: DestinoCredito<br>-fechaingreso:“Soneo.-estatus:“eehaRare StegEstatusSocioDateDate ~cuotalD:-creditolD:sou,UUIDUUID =f[> \Sejecutarcusden’“ecnaCiowe+ejecutarCuadre():DateTime BookanBoolean orPRSLFa _--] “Siaus.-fechaSolicitud:-fechaDesembolso:wealelarOuola(e: esuscrestoDatereper,Date Coola<br>“saccionesLibres(+accionesActivas(): IntegerInteger enumeration» wea +validarCobertura40():+validarFiadoresNoCruzados()Boolean<br>A} +tieneCreditoVigente(): Boolean EstatusCredito ae Boolean<br>of 4 DIFERIDAPENDIENTERPROBADAnesaoavicente a of 1 ;<br>|__Beneficiario“rome:-cedula:4‘arentesco:Stringsco:StingStr String |_Freveoiso__ee“parentesco:Proteyidodea shenString Respaldoy Restfuracién PacaooREFINANCIADO. 13 can—ea<br>cessocton cass ——<br>“echa‘festa“Raareive:RespaldoDateTineDaerineSting |. f--->} moReswalde_RESPALDORESPALDOTiponeepeldowenn wy, Desmal“atichetonpromesis:RalGarantaGarantiaCredito oneoe1 rTULaRSno<br>“aura“tenaencmsno:reger Date<br>1 -fechaPago: Date<br>“SoperFite DecaDecal<br>Acciones “Taorteat~pagada: BooleanDeca<br>“esCompra-cantidad:“euidade: IntegerBodenLotenccionesDate u “mono:“feenaPage.-estado:PagoGanencaDesirEstadoPagoDate ‘Audltoria|___ Austriavs “Binatreso Iieger<br><cunpleAnguedadUninioecha Dae) r ew ccnunanons<br>SS[mes-fotallngresosRepartibles:“fonolnctrbleeBooleanDateRepartoGanancisDesa!Decimal LZoi" | EstadoPego_PAGADA—=PENDIENTEH¥4 -nombreQuienRealiza:-entidadAfectada:-campoModificado:-valorAnterior:“ratSarn“fpoAccion: o ereSing SingTipoAuditoriaStringStringStringString -=------->) CORRECCION_OPERACIONRESTAURACION_RESPALDOeelCAMBIO_CONFIGURACIONTipoAuditoriaoleae<br>“TondoGestos Decimal<br>‘Buureeiet Decimal<br>Seeoneedatves“MaorPorscclon Decaoer<br><!-- End of picture text -->

