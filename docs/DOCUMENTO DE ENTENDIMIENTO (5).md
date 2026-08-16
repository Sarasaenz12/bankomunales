**DOCUMENTO DE ENTENDIMIENTO DEL PROYECTO**

  -----------------------------------------------------------------------
  **Nombre del proyecto:**  Bankomunales
  ------------------------- ---------------------------------------------
  **Cliente:**              Fundación Smurfit westrock

  **Versión:**              1.0

  **Creado por:**           Sara Valentina Sánchez Estrada
  -----------------------------------------------------------------------

# **1. Contexto del Negocio**

**1.1 Propósito del sistema**

Describe el problema principal que resuelve el sistema y quién lo usará.

+---------------------+------------------------------------------------+
| **Problema que      | Actualmente la administración de los           |
| resuelve**          | Bankomunales se realiza mediante formatos      |
|                     | físicos y registros manuales, lo que dificulta |
|                     | la consulta, actualización y seguimiento de la |
|                     | información.                                   |
|                     |                                                |
|                     | Además de esto, la comunidad ya no cuenta con  |
|                     | acceso al sistema BK que utilizaba             |
|                     | anteriormente, por lo que gran parte de la     |
|                     | gestión se realiza de forma manual,            |
|                     | incrementando el riesgo de pérdida de          |
|                     | información y errores en los registros. Por    |
|                     | ende se toma la decisión de crear un sistema   |
|                     | propio para la comunidad que les permita       |
|                     | centralizar la información y apoyarse de esta  |
|                     | herramienta para los procesos administrativos  |
|                     | y financieros.                                 |
+=====================+================================================+
|                     |                                                |
+---------------------+------------------------------------------------+
| **Usuarios          | Verificador, Cajero, Contable, Actualizador,   |
| principales**       | Socios, Administrador del sistema.             |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+
| **Propuesta de      | Centralizar en una única aplicación la         |
| valor**             | información de los Bankomunales para facilitar |
|                     | su administración y consulta, además,          |
|                     | optimizar la gestión de socios, acciones,      |
|                     | préstamos, pagos y demás operaciones mediante  |
|                     | la automatización de procesos, reduciendo el   |
|                     | uso de registros manuales y mejorando la       |
|                     | trazabilidad de la información.                |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+

**1.4 Contexto técnico actual**

Describe el entorno tecnológico existente del cliente (si aplica).

+---------------------+------------------------------------------------+
| **Sistemas          | Anteriormente la organización utilizaba el     |
| actuales**          | sistema BK para administrar la información de  |
|                     | los Bankomunales; actualmente ya no cuenta con |
|                     | acceso a dicha plataforma.                     |
|                     |                                                |
|                     | La gestión actual se realiza mediante formatos |
|                     | físicos, documentos impresos y registros       |
|                     | manuales utilizados durante las reuniones      |
|                     | operativas.                                    |
+=====================+================================================+
|                     |                                                |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+
| **Integraciones     | No se han identificado integraciones           |
| necesarias**        | obligatorias con sistemas externos durante la  |
|                     | etapa inicial del proyecto.                    |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+
| **Restricciones     | El sistema deberá ajustarse a las reglas de    |
| técnicas**          | negocio definidas por el cliente y             |
|                     | documentadas durante el levantamiento de       |
|                     | requisitos.                                    |
|                     |                                                |
|                     | Se desarrollará como aplicación de escritorio, |
|                     | con instalación independiente por              |
|                     | sede/computador, garantizando el aislamiento   |
|                     | de datos entre distintos Bankomunales          |
+---------------------+------------------------------------------------+
|                     |                                                |
+---------------------+------------------------------------------------+

# **2. Reglas de negocio**

  ------------------------------------------------------------------------
  **ID**      **Regla de Negocio**
  ----------- ------------------------------------------------------------
  **RN-01**   Un Bankomunal debe tener mínimo 8 y máximo 19 socios.

  **RN-02**   Ningún socio puede poseer más del 15% del total de acciones
              del Bankomunal. Esta regla aplica a partir del tercer mes de
              iniciadas las operaciones (reglamento fijo): en los primeros
              meses, con pocos socios, el tope es inalcanzable por
              aritmética. Ver D-15.

  **RN-03**   El monto de un crédito no puede superar 5 veces las acciones
              propias del socio (relación 1 a 5).

  **RN-04**   La garantía mínima combinada (socio + fiador) debe cubrir el
              40% del monto del crédito.

  **RN-05**   No se aceptan fiadores cruzados entre dos socios (que cada
              uno sea fiador del otro).

  **RN-06**   Un fiador no puede retirar o liquidar acciones comprometidas
              como garantía mientras el crédito que respaldan siga
              vigente.

  **RN-07**   Fondo para Gastos: se retiene un % de las ganancias
              mensuales, editable por cada Bankomunal. Mínimo 5% (el
              reglamento fijo dice "no menor al 5%"), recomendado 10%, que
              es el valor por defecto. Cubre los gastos operativos del
              Bankomunal: papelería, fotocopias, transporte de la Junta.

  **RN-08**   Fondo de Reserva para Incobrables: colchón de seguridad para
              cuando un socio no puede pagar su deuda y sus acciones no
              alcanzan a cubrirla. Se retiene un % de las ganancias
              mensuales, editable por cada Bankomunal: mínimo 5% (el
              reglamento fijo dice "no menor al 5%"), recomendado 10%, que
              es el valor por defecto. Tiene además un tope máximo
              respecto al capital total en acciones (valor por defecto
              20%); al alcanzarlo deja de crecer, porque ya es colchón
              suficiente. Se descuenta al absorber un incobrable (RF-36).

  **RN-09**   Autorización de venta de acciones según el PPCFC (Promedio
              de Colocación del Fondo de Crédito de los últimos 3 meses):
              menor a 80% no se vende; entre 80-90% se vende hasta 10% del
              total de acciones; entre 90-100% se vende hasta 15% del
              total.

  **RN-10**   Las ganancias de las acciones se reparten un año después de
              su compra, en el mismo mes calendario. Es una regla de todo o
              nada: si la acción se liquida antes de cumplir su año, no
              genera derecho a la ganancia de ese año en curso --- se
              devuelve sólo el valor nominal invertido. Esa ganancia no
              pagada no desaparece: queda como "ganancias no repartidas"
              en el Pasivo, en beneficio colectivo del Bankomunal. Las
              ganancias de años anteriores ya pagadas no se ven afectadas.
              Ver D-12 y D-13 en decisiones-pendientes.md.

  **RN-11**   Catálogo vigente de clases de crédito (destino del dinero),
              según el catálogo oficial "Clasificación de créditos BK
              sistema": AH (Adquisición Artículos para el Hogar), ARE
              (Adquisición o Reparación de Equipos de Trabajo), CVR
              (Construcción o Reparación de Vivienda), CV (Compra Venta de
              mercancías), ED (Educación), GP (Gastos Personales), OT
              (Otros), PR (Productivos), SL (Salud), SP (Servicios
              Públicos). Ver D-15.

  **RN-12**   El interés de cada cuota de un crédito se calcula sobre
              saldo decreciente (Tasa de interés × saldo pendiente antes
              de esa cuota), no sobre el monto original del crédito.

  **RN-13**   El valor nominal de la acción es configurable de forma
              independiente por cada Bankomunal. El reglamento fijo
              recomienda \$10.000, que es el valor por defecto del sistema.

  **RN-14**   Todo crédito debe tener al menos 1 fiador, y el fiador debe
              ser socio del Bankomunal. Es una regla fija del reglamento,
              no opcional: el máximo son 2 fiadores (RF-48) y el mínimo
              es 1.

  **RN-15**   Ningún socio puede comprar, en un mismo mes, más del 20%
              (configurable) del cupo mensual ya autorizado por el PPCFC.
              El porcentaje se calcula sobre el cupo autorizado de ese mes
              (RN-09), NO sobre el capital total del Bankomunal. Se
              diferencia de RN-02, que limita al 15% la participación
              acumulada de un socio sobre el total de acciones. Fuente:
              documento "Cargos Junta Administradora" del cliente.
  ------------------------------------------------------------------------

# **3. Requisitos Funcionales**

## **2.1 Módulo de Autenticación y Usuarios**

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-01**    Permitir el ingreso mediante una contraseña genérica,
              compartida entre todos los miembros de la junta y socios,
              sin distinción de rol.

  **RF-02**    Mostrar un mensaje de error si la contraseña es incorrecta,
              permitiendo reintentar sin límite de intentos

  **RF-03**    Permitir crear un nuevo Bankomunal la primera vez que se usa
              el sistema en un computador, mediante una clave de
              configuración inicial.

  **RF-04**    Validar que el nombre del Bankomunal no esté
              duplicado al momento de crearlo en un computador.

  **RF-05**    Si existe más de un Bankomunal creado en el mismo
              computador, mostrar una lista para seleccionar con cuál se
              va a trabajar antes de entrar al menú principal.

  **RF-06**    Si solo existe un Bankomunal creado en el computador,
              ingresar directamente a él sin mostrar pantalla de selección

  **RF-07**    Permitir volver a la pantalla de selección de Bankomunal
              desde dentro del sistema, sin necesidad de cerrar toda la
              aplicación

  **RF-08**    Garantizar que la información de cada Bankomunal permanezca
              separada dentro del mismo computador (no se mezcla entre
              Bankomunales)
  ------------------------------------------------------------------------

## **2.2 Módulo de Datos del Bankomunal** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-09**    Mostrar los Datos Generales del Bankomunal (código, nombre,
              ubicación, fecha de creación, moneda, valor nominal de la
              acción) como información de solo consulta

  **RF-10**    Calcular y mostrar automáticamente en Datos Generales:
              número de créditos otorgados, monto total en créditos
              otorgados y número de acciones vendidas, a medida que se
              registran operaciones

  **RF-11**    Mostrar en Datos del Bankomunal, como información de solo
              consulta, el saldo actual acumulado del Fondo para Gastos y
              del Fondo de Reserva para Incobrables

              Permitir editar las Condiciones de los Créditos (plazo
              máximo, tasa de interés ordinario, tasa de interés de mora,
              monto máximo de crédito)

  **RF-12**    Permitir editar el % de garantía exigido al socio y al
              fiador (RN-04).

  **RF-13**    Permitir editar los % de retención de fondos: Fondo para
              Gastos y Fondo de Reserva para Incobrables (RN-07, RN-08), y
              el tope de crecimiento de la Reserva.

              Permitir editar el valor nominal de la acción (RN-13), usado
              como base para calcular la cantidad de acciones al comprar.

  **RF-14**    Mostrar un mensaje de confirmación al guardar cambios
              exitosamente en cualquiera de estas condiciones
  ------------------------------------------------------------------------

## **2.3 Módulo de Socios**

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-15**    Permitir registrar un nuevo socio desde una pantalla
              independiente de \"Nuevo Socio\", sin necesidad de que
              exista una operación de compra de acciones

  **RF-16**    Permitir también registrar un nuevo socio automáticamente
              durante la operación de Venta de Certificados/Acciones,
              cuando la cédula ingresada no exista todavía en el sistema

  **RF-17**    Registrar los siguientes datos del socio: nombres y
              apellidos, cédula de ciudadanía, profesión u oficio,
              dirección, teléfono, celular, correo electrónico

  **RF-18**    Validar que la cédula del socio no esté duplicada dentro del
              mismo Bankomunal

  **RF-19**    Permitir consultar y actualizar los datos de un socio en
              cualquier momento, de forma independiente a las operaciones

  **RF-20**    Permitir registrar un beneficiario en caso de muerte del
              socio (nombre y cédula), a quien se cederían sus acciones.

  **RF-21**    Permitir registrar hasta 2 protegidos por socio (nombre,
              cédula, parentesco, teléfono). Se conserva únicamente el
              registro de las personas; el Fondo de Protección que lo
              respaldaba quedó fuera del alcance (ver D-05).
  ------------------------------------------------------------------------

##  **2.4 Módulo de Acciones (VC)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-22**    Registrar la compra de Certificados/Acciones (VC) a un socio
              (nuevo o existente), guardando fecha, monto y actualizando
              su saldo de acciones

  **RF-23**    Calcular la cantidad de acciones a partir del monto
              ingresado, usando el valor nominal de la acción configurado
              en Datos del Bankomunal (RN-13)

  **RF-24**    Calcular y mostrar el % de participación del socio sobre el
              total de acciones del Bankomunal después de la compra

  **RF-25**    Validar el cumplimiento de RN-02 (tope del 15%) y bloquear o
              alertar si la compra lo excede.

  **RF-26**    Calcular el PPCFC (RN-09) para determinar si se autoriza la
              venta de acciones ese mes y hasta qué cantidad.

  **RF-27**    Registrar el mes de compra de cada acción, como insumo para
              el cálculo de Ganancias Repartidas (RN-10).
  ------------------------------------------------------------------------

## **2.5 Módulo de Liquidación de Acciones (LC)**

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-28**    Permitir liquidar acciones de un socio de forma parcial
              (sigue activo) o total (retiro completo del Bankomunal)

  **RF-29**    Calcular cuántas acciones del socio están libres,
              descontando las que estén comprometidas como garantía de un
              crédito vigente (RN-06)

  **RF-30**    Bloquear la liquidación de acciones que estén comprometidas
              en garantía mientras el crédito que respaldan siga vigente
              (RN-06)

  **RF-31**    Calcular el valor total a favor del socio: valor nominal de
              las acciones a liquidar, más las ganancias de años ya
              cumplidos que estén devengadas y todavía sin pagar. NO se
              incluye la ganancia del año en curso si las acciones aún no
              cumplen su aniversario (RN-10); esa queda como ganancia
              colectiva. Las ganancias de años anteriores ya pagadas no se
              recalculan ni se recuperan. Ver D-13.

  **RF-32**    Calcular el saldo de deuda propia del socio, si tiene
              crédito vigente (capital + intereses ordinarios + intereses
              de mora)

  **RF-33**    Si el socio no tiene deuda, pagar directamente el valor a
              favor calculado en RF-31 (nominal + ganancias ya vencidas y
              sin pagar)

  **RF-34**    (RET): Si el socio tiene deuda menor al valor de sus
              acciones + ganancias, descontar la deuda y pagarle la
              diferencia

  **RF-35**    (RET): Si el socio tiene deuda mayor al valor de sus
              acciones + ganancias, aplicar todo ese valor a la deuda y
              trasladar el monto restante a la cuenta de Incobrables

  **RF-36**    Al trasladar un monto a Incobrables, descontarlo
              automáticamente de la Reserva por Incobrables configurada en
              Datos del Bankomunal

  **RF-37**    Si la liquidación es total, actualizar el estatus del socio
              a \"Retirado Voluntario\" (sin deuda) o \"Retirado con
              Deuda\" (si quedó saldo en Incobrables)

  **RF-38**    Registrar la fecha de retiro cuando la liquidación sea total
  ------------------------------------------------------------------------

## **2.6 Módulo de Ganancias Repartidas (UR)**

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-39**    Devengar cada mes, a cada lote de acciones activo, la
              ganancia del mes (cantidad de acciones × valor de ganancia
              por acción), acumulándola sin pagarla; e identificar cuáles
              lotes cumplen 1 año desde su compra (RN-10) para pagar la
              ganancia acumulada. Ver D-12 en decisiones-pendientes.md.

  **RF-40**    Calcular el \"Total Ingresos Repartibles\" del mes, sumando
              Intereses Ordinarios + Intereses de Mora + Otros Ingresos.

  **RF-41**    Restar del total anterior los montos apartados para Fondo de
              Incobrables y Fondo de Gastos, para obtener el Balance de
              Gestión Mensual Neto. (El Fondo de Protección quedó fuera del
              alcance --- aplicaba solo a los Bankomunales de Venezuela; ver
              D-05 en decisiones-pendientes.md.)

  **RF-42**    Calcular la ganancia a pagar a cada socio (acciones que
              cumplen año × valor de ganancia por acción).

              Calcular el valor de ganancia por acción del mes

              Registrar el pago de la ganancia repartida a cada socio, con
              fecha y monto.
  ------------------------------------------------------------------------

## **2.7 Módulo de Solicitud y Aprobación de Crédito** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-43**    Permitir registrar una Solicitud de Crédito desde una
              pantalla independiente \"Nuevo Crédito\", capturando datos
              del solicitante, ingresos/egresos y destino del crédito

  **RF-44**    Permitir también que el Desembolso se registre directamente
              sin pasar por una solicitud digital previa (igual que el
              sistema anterior), para cuando la solicitud se maneje en
              papel

  **RF-45**    Calcular automáticamente la Capacidad de Pago del
              solicitante (Total Ingresos − Total Egresos)

  **RF-46**    Calcular automáticamente la tabla del crédito usando el
              modelo de saldo decreciente (RN-12): interés de cada cuota
              = Tasa × saldo pendiente antes de esa cuota; capital fijo
              = Monto ÷ Nº de cuotas. Mostrar como vista previa: Monto
              Total a Pagar, Cuota Mensual, Capital por Cuota e Interés
              por Cuota

  **RF-47**    Registrar el destino del crédito según el catálogo de clases
              vigente (RN-11).

  **RF-48**    Registrar hasta 2 fiadores por solicitud, con su cédula y
              número de acciones comprometidas en garantía

  **RF-49**    Validar que el socio y sus fiadores tengan acciones
              suficientes para cubrir la garantía mínima exigida (RN-04)

  **RF-50**    Permitir que la Junta registre la decisión sobre la
              solicitud: Aprobado, Modificado, Negado o Diferido

  **RF-51**    Si la solicitud queda Diferida, permitir dejar una
              observación y mantenerla visible hasta que se resuelva

  **RF-52**    Si la solicitud es aprobada, permitir pasar directo a
              registrar el Desembolso con los datos ya cargados
  ------------------------------------------------------------------------

## **2.8 Módulo de Desembolso de Crédito (CON)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-53**    Asignar automáticamente el siguiente número de crédito en
              secuencia

  **RF-54**    Precargar los datos si el desembolso proviene de una
              Solicitud aprobada, o permitir cargarlos manualmente si no
              la hubo

  **RF-55**    Validar que el monto del crédito no supere el Monto Máximo
              configurado en Datos del Bankomunal

  **RF-56**    Validar el cumplimiento de RN-03 (relación 1 a 5 respecto a
              las acciones propias del socio).

  **RF-57**    Validar el cumplimiento de RN-04 (garantía mínima combinada
              del 40%).

  **RF-58**    Validar el cumplimiento de RN-05 (no fiadores cruzados).

  **RF-59**    Precargar las tasas de interés ordinario y de mora desde
              Datos del Bankomunal

  **RF-60**    Registrar la Clase de Crédito (RN-11), frecuencia de pago y
              fecha de vencimiento

  **RF-61**    Calcular automáticamente la tabla de pagos usando el modelo
              de saldo decreciente (RN-12), de forma consistente con el
              módulo de Solicitud.

  **RF-62**    Registrar el desembolso en el Libro de Ingresos y Egresos,
              actualizando el saldo
  ------------------------------------------------------------------------

## **2.9 Módulo de Servicio de la deuda**

###  **2.9.1 Intereses Ordinarios (OR)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-63**    Permitir seleccionar el Nº de Crédito y mostrar los datos
              del socio para confirmar antes de registrar

  **RF-64**    Calcular automáticamente el interés de la cuota según RN-12
              (Tasa de interés × saldo pendiente antes de esa cuota).

  **RF-65**    Registrar el monto de interés ordinario efectivamente
              pagado, actualizando el Libro de Ingresos y Egresos
  ------------------------------------------------------------------------

### **2.9.2 Módulo de Pago de Cuota (PC)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-66**    Permitir seleccionar el Nº de Crédito y mostrar los datos
              del socio para confirmar antes de registrar

  **RF-67**    Calcular automáticamente el capital de la cuota (Monto del
              crédito ÷ Número de cuotas, capital fijo).

  **RF-68**    Validar que el monto pagado no sea mayor al saldo pendiente
              del crédito.

  **RF-69**    Actualizar automáticamente el saldo del crédito al registrar
              el pago
  ------------------------------------------------------------------------

### **2.9.3 Módulo de Intereses de Mora (MO)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-70**    Permitir seleccionar el Nº de Crédito y mostrar los datos
              del socio para confirmar antes de registrar

  **RF-71**    Calcular automáticamente el monto de mora según los días de
              atraso y la Tasa de Mora configurada

  **RF-72**    Registrar el monto de mora efectivamente pagado,
              actualizando el Libro de Ingresos y Egresos
  ------------------------------------------------------------------------

## **2.10 Módulo de Pago de Deuda Pendiente (PDP)** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-73**    Permitir buscar el crédito de un socio retirado con deuda
              pendiente en Incobrables

  **RF-74**    Permitir registrar uno o varios pagos/abonos hasta saldar la
              deuda

  **RF-75**    Reflejar el monto pagado como \"Otros Ingresos\" en el
              Informe de Gestión, tras el Cuadre y Cierre de Mes

  **RF-76**    Si el pago cancela toda la deuda, actualizar el estatus del
              socio a \"Retirado con Deuda Pagada\" y su deuda a cero
  ------------------------------------------------------------------------

## **2.11 Módulo de Refinanciamiento (COR)**

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-77**    Permitir seleccionar un crédito vigente para refinanciar,
              mostrando su saldo de capital pendiente

  **RF-78**    Permitir decidir si los intereses ordinarios y de mora
              pendientes se pagan en efectivo en ese momento, o se
              incluyen dentro del nuevo crédito

  **RF-79**    Registrar el cierre contable del crédito anterior (pago
              total de su saldo)

  **RF-80**    Abrir un nuevo crédito precargado con los datos del anterior
              (monto, socio, fiadores), permitiendo modificar monto,
              tasas, fiadores y fecha de vencimiento

  **RF-81**    Si se incluyen los intereses pendientes, sumarlos
              automáticamente al monto del nuevo crédito

  **RF-82**    Calcular la nueva tabla de pagos del crédito refinanciado
              usando el modelo de saldo decreciente (RN-12)
  ------------------------------------------------------------------------

## **2.12 Módulo de Caja y Contabilidad** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-83**    (OI): Registrar Otros Ingresos, con fecha y monto

  **RF-84**    (EG):Registrar Otros Egresos, con fecha y monto

  **RF-85**    (IFG):Registrar Ingreso al Fondo para Gastos, sumándolo al
              saldo acumulado de ese fondo

  **RF-86**    (GBK): Registrar Gastos del Bankomunal, descontándolos del
              saldo acumulado del Fondo para Gastos

  **RF-87**    Registrar Donaciones, ingresándolas automáticamente como
              Ingreso al Fondo para Gastos

  **RF-88**    Registrar Bienes Adquiridos (propios o en comodato) como
              Activo Fijo, sin afectar el saldo de caja

  **RF-89**    Permitir corregir cualquier operación del Libro de Ingresos
              y Egresos antes del Cierre de Mes

  **RF-90**    Permitir corregir operaciones también después del Cierre de
              Mes, registrando quién hizo la corrección, cuándo y el
              motivo (auditoría)
  ------------------------------------------------------------------------

## **2.13 Módulo de Cuadre y Cierre de Mes** 

  ------------------------------------------------------------------------
  **ID**      **El sistema debe\...**
  ----------- ------------------------------------------------------------
  **RF-91**    Permitir ejecutar el Proceso de Cuadre las veces que sea
              necesario, sin afectar los datos definitivos

  **RF-92**    Generar el Informe de Gestión Mensual (socios, créditos,
              rendimiento estimado, disponibilidad de efectivo)

  **RF-93**    Generar el Balance del Mes, comparando Activo vs. Pasivo

  **RF-94**    Impedir cerrar un mes si el mes anterior sigue sin cerrar

  **RF-95**    Al ejecutar el Cierre Mensual, calcular y actualizar
              automáticamente los saldos del Fondo de Gastos y del Fondo
              de Incobrables según los % configurados

  **RF-96**    Al ejecutar el Cierre Mensual, recalcular el PPCFC (RN-09)
              incluyendo el mes cerrado.

  **RF-97**    Permitir corregir operaciones después del Cierre Mensual,
              quedando registrado quién, cuándo y por qué (según la
              auditoría ya definida)
  ------------------------------------------------------------------------

## **2.14 Módulo de Reportes** 

  -------------------------------------------------------------------------
  **ID**       **El sistema debe\...**
  ------------ ------------------------------------------------------------
  **RF-98**     Generar el Reporte de Socios Activos (certificados activos,
               liquidados parcialmente, fecha de ingreso)

  **RF-99**     Generar el Reporte de Gestión (movimiento del mes y
               acumulado histórico)

  **RF-100**    Generar el Balance General por mes cerrado

  **RF-101**    Generar el Reporte de Fiadores (créditos avalados,
               certificados comprometidos, % garantizado)

  **RF-102**    Generar el reporte de Saldos de Créditos Vencidos

  **RF-103**    Generar el reporte de Saldos de Créditos Vigentes

  **RF-104**    Generar el Libro de Ingresos y Egresos filtrable por rango
               de fechas

  **RF-105**    Generar el Control de Acciones por Socio, detallado mes a
               mes

  **RF-106**    Generar el Reporte de Rendimiento de las Acciones, por rango
               de fechas seleccionado por el usuario

  **RF-107**    Permitir exportar o imprimir cualquiera de los reportes
               anteriores
  -------------------------------------------------------------------------

## **2.15 Módulo de Respaldo y Restauración** 

  -------------------------------------------------------------------------
  **ID**       **El sistema debe\...**
  ------------ ------------------------------------------------------------
  **RF-108**    Generar un archivo de respaldo con toda la información del
               Bankomunal, en una ubicación elegida por el usuario
               (PenDrive, carpeta local, adjunto para correo)

  **RF-109**    Permitir restaurar un archivo de respaldo previamente
               generado, ubicándolo manualmente en el computador

  **RF-110**    Antes de restaurar, alertar claramente que esta acción puede
               sobrescribir la información actual del Bankomunal

  **RF-111**    Validar que el archivo a restaurar corresponda al Bankomunal
               correcto
  -------------------------------------------------------------------------

## **2.16 Módulo de Auditoría** 

  -------------------------------------------------------------------------
  **ID**       **El sistema debe\...**
  ------------ ------------------------------------------------------------
  **RF-112**    Registrar en una bitácora de Auditoría todo cambio a la
               Configuración del Bankomunal (tasas, % de garantía, % de
               fondos, valor nominal), capturando: quién lo hizo, fecha,
               campo modificado, valor anterior y valor nuevo.

  **RF-113**    Al guardar un cambio sensible (Configuración, corrección de
               operación tras cierre, restauración de respaldo, borrado de
               Bankomunal), solicitar al usuario que indique su nombre
               antes de confirmar --- este campo es solo para trazabilidad,
               no es una validación de seguridad ni bloquea por rol.

  **RF-114**    Permitir consultar el historial completo de Auditoría,
               filtrable por fecha o tipo de cambio.
  -------------------------------------------------------------------------

# **3. Requisitos No Funcionales** 

  --------------------------------------------------------------------------
  **ID**       **Descripción**
  ------------ -------------------------------------------------------------
  **RNF-01**   El sistema debe responder de forma inmediata al registrar
               operaciones, al ser una base de datos local sin dependencia
               de red.

  **RNF-02**   La contraseña genérica debe almacenarse cifrada, no en texto
               plano, aunque sea compartida entre todos los usuarios.

  **RNF-03**   El sistema debe funcionar completamente sin conexión a
               internet, ya que opera de forma local en el computador de
               cada Bankomunall.

  **RNF-04**   La interfaz debe ser simple y clara para personas con
               conocimientos básicos de computación y aritmética (perfil de
               los socios/junta), sin necesidad de capacitación técnica
               extensa.

  **RNF-05**   El sistema debe funcionar en computadores de
               escritorio/portátiles con Windows, incluyendo equipos de gama
               modesta (no se requiere hardware reciente).

  **RNF-06**   El proceso de respaldo y restauración debe completarse sin
               pérdida de información y sin depender de un servicio externo
               (Fundefir ya no está disponible).

  **RNF-07**   Los datos de cada Bankomunal deben permanecer completamente
               aislados de los demás, incluso cuando coexisten en el mismo
               computador

  **RNF-08**   Todo cambio a la Configuración del Bankomunal, corrección de
               operaciones (antes o después del cierre de mes),
               respaldo/restauración de datos, y borrado de un Bankomunal
               debe quedar registrado en la bitácora de Auditoría con
               usuario, fecha y motivo. Métrica: 100% de acciones sensibles
               con registro de auditoría.

  **RNF-09**   El proceso de instalación debe poder ser realizado por una
               persona sin conocimientos técnicos avanzados, siguiendo un
               asistente guiado (similar al sistema anterior).

  **RNF-10**   El sistema debe soportar cómodamente el máximo de socios
               permitido por Bankomunal, y permitir múltiples Bankomunales
               en un mismo computador sin degradar el rendimiento.

  **RNF-11**   El sistema debe permitir corregir errores de configuración o
               de reglas de negocio (RN) sin requerir reinstalación ni
               intervención externa.
  --------------------------------------------------------------------------

# 

# 

# **4. Casos de Uso**

## **1.1 Acciones por Actor**

  -----------------------------------------------------------------------
  **Actor**            **Acciones / Funciones principales en el sistema**
  -------------------- --------------------------------------------------
  **Verificador**      1\. Iniciar sesión 2. Revisar y decidir sobre
                       solicitudes de crédito
                       (aprobar/modificar/negar/diferir) 3. Verificar
                       acciones y garantías del socio/fiador antes de una
                       operación 4. Calcular y autorizar la venta de
                       acciones del mes (PPCFC) 5. Verificar créditos
                       aprobados antes del desembolso 6. Calcular el
                       saldo de deuda de un socio al liquidar/retirar 7.
                       Consultar el reporte de fiadores

  **Cajero**           1\. Iniciar sesión 2. Recibir/entregar efectivo en
                       cada operación 3. Registrar el pago de compra de
                       acciones 4. Registrar el desembolso de un crédito
                       5. Registrar el pago de cuotas (capital + interés)
                       6. Registrar Otros Ingresos y Otros Egresos 7.
                       Registrar Gastos del Bankomunal 8. Participar en
                       el Proceso de Cuadre de caja

  **Contable**         1\. Iniciar sesión 2. Registrar las operaciones en
                       el Libro de Ingresos y Egresos 3. Registrar el
                       Pago de Deuda Pendiente de socios retirados 4.
                       Ejecutar el Cierre Mensual 5. Generar el Balance
                       General y el Informe de Gestión 6. Corregir
                       operaciones (antes o después del cierre, con
                       auditoría)

  **Actualizador**     1\. Iniciar sesión 2. Registrar y actualizar los
                       datos de los socios 3. Actualizar el Control de
                       Acciones por socio 4. Actualizar las acciones
                       comprometidas en garantía del fiador 5. Actualizar
                       el estatus del socio (activo/retirado) 6. Generar
                       reportes de Control de Acciones y Rendimiento de
                       Acciones

  **Socio del          1\. Comprar acciones 2. Solicitar un crédito 3.
  Bankomunal**         Realizar pagos de cuota 4. Actuar como fiador de
                       otro socio 5. Liquidar o retirar sus acciones 6.
                       Consultar su información personal y su historial

  **Administrador del  1\. Crear un nuevo Bankomunal 2. Configurar los
  Sistema**            Datos del Bankomunal (tasas, %, valor nominal,
                       garantías) 3. Generar y restaurar respaldos de
                       información 4. Autorizar acciones sensibles
                       (restauración, borrado de Bankomunal)
  -----------------------------------------------------------------------

## 

## 

## **1.2 Descripción Detallada de Casos de Uso**

**Caso de Uso: CU-01 --- Iniciar Sesión**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-01                                           |
+====================+=================================================+
| **Nombre:**        | Iniciar Sesión                                  |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | La aplicación debe estar instalada en el        |
|                    | computador. Debe existir al menos un Bankomunal |
|                    | ya creado (si es el primer uso, el actor debe   |
|                    | seguir primero el CU-02 Crear Bankomunal).      |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario abre la aplicación.              |
| principal:**       |                                                 |
|                    | 2.  El sistema muestra la pantalla de ingreso   |
|                    |     > solicitando la contraseña genérica.       |
|                    |                                                 |
|                    | 3.  El usuario digita la contraseña y confirma. |
|                    |                                                 |
|                    | 4.  El sistema valida la contraseña y, si es    |
|                    |     > correcta, verifica cuántos Bankomunales   |
|                    |     > existen en el computador.                 |
|                    |                                                 |
|                    | 5.  Si existe más de un Bankomunal, el sistema  |
|                    |     > muestra la lista para que el usuario      |
|                    |     > seleccione con cuál va a trabajar.        |
|                    |                                                 |
|                    | 6.  El usuario selecciona el Bankomunal.        |
|                    |                                                 |
|                    | 7.  El sistema confirma el ingreso y muestra el |
|                    |     > Menú Principal del Bankomunal             |
|                    |     > seleccionado.                             |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si la contraseña ingresada es incorrecta,   |
| alternativo:**     |     > el sistema muestra un mensaje de error y  |
|                    |     > permite reintentar sin límite de intentos |
|                    |     > (RF-02).                                  |
|                    |                                                 |
|                    | -   Si en el computador solo existe un          |
|                    |     > Bankomunal creado, el sistema omite el    |
|                    |     > paso 5-6 y entra directamente a su Menú   |
|                    |     > Principal (RF-06).                        |
|                    |                                                 |
|                    | -   Si no existe ningún Bankomunal creado       |
|                    |     > todavía, el sistema redirige al CU-02     |
|                    |     > (Crear Bankomunal) en vez de mostrar el   |
|                    |     > Menú Principal.                           |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El usuario queda autenticado y ubicado en el    |
|                    | Menú Principal del Bankomunal seleccionado, con |
|                    | acceso a todos los módulos del sistem           |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-02 --- Crear Bankomunal**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-02                                           |
+====================+=================================================+
| **Nombre:**        | Crear Bankomunal                                |
+--------------------+-------------------------------------------------+
| **Actor            | Administrador del Sistema (Todos)               |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | La aplicación debe estar instalada en el        |
|                    | computador.                                     |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario abre la aplicación por primera   |
| principal:**       |     > vez                                       |
|                    |                                                 |
|                    | 2.  El sistema solicita la clave de             |
|                    |     > configuración inicial.                    |
|                    |                                                 |
|                    | 3.  El usuario ingresa la clave y confirma.     |
|                    |                                                 |
|                    | 4.  El usuario accede a la opción de crear un   |
|                    |     > nuevo Bankomunal.                         |
|                    |                                                 |
|                    | 5.  El sistema muestra el formulario con los    |
|                    |     > Datos Generales (nombre, código,          |
|                    |     > ubicación, moneda, valor nominal de la    |
|                    |     > acción).                                  |
|                    |                                                 |
|                    | 6.  El usuario diligencia los datos y guarda.   |
|                    |                                                 |
|                    | 7.  El sistema confirma la creación del         |
|                    |     > Bankomunal.                               |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el nombre/código ya existe en el mismo   |
| alternativo:**     |     > computador, el sistema muestra un mensaje |
|                    |     > de error y solicita corregirlo.           |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El Bankomunal queda creado y disponible para    |
|                    | seleccionarse en el inicio de sesión (CU-01).   |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-03 --- Seleccionar Bankomunal Activo**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-03                                           |
+====================+=================================================+
| **Nombre:**        | Seleccionar Bankomunal Activo                   |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir más de un Bankomunal creado en el  |
|                    | computador; el usuario ya inició sesión         |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El sistema muestra la lista de Bankomunales |
| principal:**       |     > disponibles.                              |
|                    |                                                 |
|                    | 2.  El usuario selecciona el Bankomunal con el  |
|                    |     > que desea trabajar.                       |
|                    |                                                 |
|                    | 3.  El sistema carga los datos de ese           |
|                    |     > Bankomunal y muestra su Menú Principal.   |
|                    |                                                 |
|                    | 4.  El usuario puede volver a esta pantalla en  |
|                    |     > cualquier momento sin cerrar la           |
|                    |     > aplicación.                               |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si solo existe un Bankomunal, el sistema    |
| alternativo:**     |     > entra directo a él, sin ejecutar este     |
|                    |     > caso de uso.                              |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El usuario queda ubicado en el Bankomunal       |
|                    | seleccionado, con sus datos aislados de         |
|                    | cualquier otro en el mismo computador.          |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-04 --- Configurar Datos del Bankomunal**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-04                                           |
+====================+=================================================+
| **Nombre:**        | Configurar Datos del Bankomunal                 |
+--------------------+-------------------------------------------------+
| **Actor            | Administrador del Sistema (Todos)               |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El Bankomunal debe existir y el usuario debe    |
|                    | estar ubicado en él banco creado.               |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra al módulo \"Datos del      |
| principal:**       |     > Bankomunal\".                             |
|                    |                                                 |
|                    | 2.  El sistema muestra los Datos Generales      |
|                    |     > (solo consulta), las condiciones de       |
|                    |     > crédito, % de garantía, % de retención de |
|                    |     > fondos y valor nominal de la acción       |
|                    |     > (editables).                              |
|                    |                                                 |
|                    | 3.  El usuario presiona en editar valores       |
|                    |                                                 |
|                    | 4.  El usuario modifica el/los campos que       |
|                    |     > necesita.                                 |
|                    |                                                 |
|                    | 5.  El usuario guarda los cambios.              |
|                    |                                                 |
|                    | 6.  El sistema solicita el nombre de quien      |
|                    |     > realiza el cambio y registra la           |
|                    |     > modificación en Auditoría.                |
|                    |                                                 |
|                    | 7.  El sistema confirma el guardado exitoso.    |
+--------------------+-------------------------------------------------+
| **Flujo            | Si el usuario sólo desea consultar los saldos   |
| alternativo:**     | acumulados de los fondos, los visualiza sin     |
|                    | realizar cambios.                               |
+--------------------+-------------------------------------------------+
| **Postcondición:** | -   Las condiciones configurables del           |
|                    |     > Bankomunal quedan actualizadas y se       |
|                    |     > aplican a las siguientes operaciones.     |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-05 --- Registrar Nuevo Socio**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-05                                           |
+====================+=================================================+
| **Nombre:**        | Registrar Nuevo Socio                           |
+--------------------+-------------------------------------------------+
| **Actor            | Actualizador                                    |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El usuario debe estar ubicado en un Bankomunal  |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra a la pantalla de "Socios"  |
| principal:**       |                                                 |
|                    | 2.  El usuario presiona el botón de \"Nuevo     |
|                    |     > Socio\".                                  |
|                    |                                                 |
|                    | 3.  El sistema solicita los datos del socio     |
|                    |     > (nombres, cédula, profesión, dirección,   |
|                    |     > teléfono, celular, correo).               |
|                    |                                                 |
|                    | 4.  El usuario diligencia los datos, y          |
|                    |     > opcionalmente el beneficiario en caso de  |
|                    |     > muerte y hasta 2 protegidos.              |
|                    |                                                 |
|                    | 5.  El usuario guarda el registro.              |
|                    |                                                 |
|                    | 6.  El sistema valida que la cédula no esté     |
|                    |     > duplicada y crea el registro.             |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el registro se origina durante una       |
| alternativo:**     |     > Compra de Acciones (CU-07) a una cédula   |
|                    |     > nueva, el sistema activa este mismo       |
|                    |     > formulario automáticamente.               |
|                    |                                                 |
|                    | ```{=html}                                      |
|                    | <!-- -->                                        |
|                    | ```                                             |
|                    | -   Si la cédula ya existe, el sistema muestra  |
|                    |     > un error y no permite duplicar el         |
|                    |     > registro.                                 |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El socio queda registrado y disponible para     |
|                    | realizar operaciones.                           |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-06 --- Consultar/Actualizar Datos de Socio**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-06                                           |
+====================+=================================================+
| **Nombre:**        | Consultar/Actualizar Datos de Socio             |
+--------------------+-------------------------------------------------+
| **Actor            | Actualizador                                    |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El socio debe existir en el sistema.            |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario busca al socio por nombre o      |
| principal:**       |     > cédula.                                   |
|                    |                                                 |
|                    | 2.  El sistema muestra sus datos personales,    |
|                    |     > protegidos/beneficiario y su historial    |
|                    |     > (acciones, créditos).                     |
|                    |                                                 |
|                    | 3.  El usuario modifica el/los datos            |
|                    |     > necesarios.                               |
|                    |                                                 |
|                    | 4.  El usuario guarda los cambios.              |
|                    |                                                 |
|                    | 5.  El sistema actualiza el registro.           |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el usuario solo desea consultar, sale de |
| alternativo:**     |     > la pantalla sin guardar cambios.          |
+--------------------+-------------------------------------------------+
| **Postcondición:** | Si el usuario solo desea consultar, sale de la  |
|                    | pantalla sin guardar cambios.                   |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-07 --- Comprar Acciones**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-07                                           |
+====================+=================================================+
| **Nombre:**        | Comprar Acciones                                |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El socio debe existir o registrarse en el       |
|                    | momento); debe existir autorización de venta    |
|                    | para el mes.                                    |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El socio informa cuánto desea invertir.     |
| principal:**       |                                                 |
|                    | 2.  El Verificador revisa el PPCFC del mes que  |
|                    |     > da el sistema el cual ya está configurado |
|                    |     > para dar el balance de cómo está el PPCFC |
|                    |     > y confirma si hay autorización de venta y |
|                    |     > hasta qué cantidad.                       |
|                    |                                                 |
|                    | 3.  El sistema calcula la cantidad de acciones  |
|                    |     > según el valor nominal y el nuevo % de    |
|                    |     > participación del socio.                  |
|                    |                                                 |
|                    | 4.  El sistema valida que no se supere el tope  |
|                    |     > del 15%.                                  |
|                    |                                                 |
|                    | 5.  El Cajero recibe el dinero y registra la    |
|                    |     > compra.                                   |
|                    |                                                 |
|                    | 6.  El sistema registra el mes de compra de la  |
|                    |     > acción y actualiza el saldo del socio.    |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el PPCFC no autoriza venta ese mes, el   |
| alternativo:**     |     > sistema bloquea el registro de nuevas     |
|                    |     > compras.                                  |
|                    |                                                 |
|                    | -   Si la compra supera el 15% permitido, el    |
|                    |     > sistema alerta o bloquea la operación.    |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El socio queda con sus acciones actualizadas,   |
|                    | disponibles para el cálculo de ganancias        |
|                    | futuras.                                        |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-08 --- Liquidar Acciones (parcial o total)**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-08                                           |
+====================+=================================================+
| **Nombre:**        | Liquidar Acciones                               |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El socio debe tener acciones activas.           |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El socio solicita liquidar acciones         |
| principal:**       |     > (parcial o total).                        |
|                    |                                                 |
|                    | 2.  El Verificador calcula las acciones libres  |
|                    |     > del socio, descontando las comprometidas  |
|                    |     > en garantía.                              |
|                    |                                                 |
|                    | 3.  El sistema calcula el valor total a favor   |
|                    |     > del socio (acciones + ganancias) y su     |
|                    |     > saldo de deuda propia, si tiene crédito   |
|                    |     > vigente.                                  |
|                    |                                                 |
|                    | 4.  El sistema determina el escenario: sin      |
|                    |     > deuda, con deuda menor, o con deuda       |
|                    |     > mayor.                                    |
|                    |                                                 |
|                    | 5.  El Cajero realiza el pago o cobro           |
|                    |     > correspondiente.                          |
|                    |                                                 |
|                    | 6.  El Actualizador actualiza el registro del   |
|                    |     > socio y, si es liquidación total, su      |
|                    |     > estatus.                                  |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si las acciones están comprometidas en      |
| alternativo:**     |     > garantía de un crédito vigente, el        |
|                    |     > sistema bloquea la operación.             |
|                    |                                                 |
|                    | -   Si la deuda supera el valor disponible, el  |
|                    |     > sistema traslada el excedente a           |
|                    |     > Incobrables y descuenta la Reserva        |
|                    |     > correspondiente.                          |
+--------------------+-------------------------------------------------+
| **Postcondición:** | Las acciones del socio quedan liquidadas        |
|                    | (parcial o totalmente) y su estatus actualizado |
|                    | si aplica.                                      |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-09 --- Repartir Ganancias**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-09                                           |
+====================+=================================================+
| **Nombre:**        | Repartir Ganancias                              |
+--------------------+-------------------------------------------------+
| **Actor            | Verificador / Actualizador                      |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un Balance de Gestión Mensual      |
|                    | calculado.                                      |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El sistema identifica las acciones que      |
| principal:**       |     > cumplen 1 año ese mes.                    |
|                    |                                                 |
|                    | 2.  El sistema calcula el Total de Ingresos     |
|                    |     > Repartibles y resta los fondos            |
|                    |     > correspondientes para obtener el Balance  |
|                    |     > Neto.                                     |
|                    |                                                 |
|                    | 3.  El sistema calcula el valor de ganancia por |
|                    |     > acción del mes.                           |
|                    |                                                 |
|                    | 4.  El sistema calcula la ganancia a pagar a    |
|                    |     > cada socio.                               |
|                    |                                                 |
|                    | 5.  El usuario registra el pago de la ganancia  |
|                    |     > repartida a cada socio.                   |
+--------------------+-------------------------------------------------+
| **Flujo            |                                                 |
| alternativo:**     |                                                 |
+--------------------+-------------------------------------------------+
| **Postcondición:** | Las ganancias del periodo quedan repartidas y   |
|                    | registradas para cada socio beneficiado.        |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-10 --- Solicitar Crédito**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-10                                           |
+====================+=================================================+
| **Nombre:**        | Solicitar Crédito                               |
+--------------------+-------------------------------------------------+
| **Actor            | Socio, Verificador                              |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El socio debe estar activo y sin sanciones      |
|                    | vigentes.                                       |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El socio solicita un crédito desde la       |
| principal:**       |     > pantalla \"Nuevo Crédito\" o en papel.    |
|                    |                                                 |
|                    | 2.  El sistema captura los datos del            |
|                    |     > solicitante, ingresos/egresos y destino   |
|                    |     > del crédito.                              |
|                    |                                                 |
|                    | 3.  El sistema mira que se cumpla relación 1 a  |
|                    |     > 5 (RN-03)                                 |
|                    |                                                 |
|                    | 4.  El sistema calcula la Capacidad de Pago y   |
|                    |     > la tabla del crédito con el modelo de     |
|                    |     > saldo decreciente.                        |
|                    |                                                 |
|                    | 5.  El socio registra hasta 2 fiadores con sus  |
|                    |     > acciones en garantía.                     |
|                    |                                                 |
|                    | 6.  El sistema valida que se cumpla la garantía |
|                    |     > mínima.                                   |
|                    |                                                 |
|                    | 7.  La solicitud queda lista para revisión de   |
|                    |     > la Junta                                  |
+--------------------+-------------------------------------------------+
| **Flujo            | Si la solicitud se maneja en papel, el proceso  |
| alternativo:**     | continúa directo en el Desembolso, sin pasar    |
|                    | por este caso de uso.                           |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La solicitud de crédito queda registrada y      |
|                    | lista para su aprobación.                       |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-11 --- Aprobar / Negar / Diferir Solicitud de
Crédito**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-11                                           |
+====================+=================================================+
| **Nombre:**        | Aprobar / Negar / Diferir Solicitud de Crédito  |
+--------------------+-------------------------------------------------+
| **Actor            | Junta Administradora                            |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir una solicitud de crédito           |
|                    | registrada                                      |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  La Junta revisa la solicitud y sus          |
| principal:**       |     > validaciones (capacidad de pago,          |
|                    |     > garantías).                               |
|                    |                                                 |
|                    | 2.  La Junta registra su decisión: Aprobado,    |
|                    |     > Modificado, Negado o Diferido.            |
|                    |                                                 |
|                    | 3.  Si es Diferido, el usuario agrega una       |
|                    |     > observación.                              |
|                    |                                                 |
|                    | 4.  Si es Aprobado, el sistema permite pasar    |
|                    |     > directo al Desembolso con los datos       |
|                    |     > precargados.                              |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si la solicitud es Negada, el sistema       |
| alternativo:**     |     > archiva la decisión sin continuar el      |
|                    |     > flujo de crédito.                         |
|                    |                                                 |
|                    | -   Si es Diferida, la solicitud permanece      |
|                    |     > visible hasta que se resuelva la          |
|                    |     > observación.                              |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La solicitud queda con una decisión registrada. |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-12 --- Desembolsar Crédito**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-12                                           |
+====================+=================================================+
| **Nombre:**        | Desembolsar Crédito                             |
+--------------------+-------------------------------------------------+
| **Actor            | Cajero/Todos                                    |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | La solicitud debe estar aprobada, o el          |
|                    | desembolso se registra directamente si no hubo  |
|                    | solicitud digital.                              |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El sistema asigna el siguiente número de    |
| principal:**       |     > crédito.                                  |
|                    |                                                 |
|                    | 2.  El sistema precarga los datos de la         |
|                    |     > solicitud aprobada, o el usuario los      |
|                    |     > carga manualmente.                        |
|                    |                                                 |
|                    | 3.  El sistema valida el monto máximo, la       |
|                    |     > relación 1 a 5, la garantía del 40% y que |
|                    |     > no haya fiadores cruzados.                |
|                    |                                                 |
|                    | 4.  El sistema precarga las tasas configuradas  |
|                    |     > y registra clase de crédito, frecuencia y |
|                    |     > fecha de vencimiento.                     |
|                    |                                                 |
|                    | 5.  El sistema calcula la tabla de pagos con el |
|                    |     > modelo de saldo decreciente.              |
|                    |                                                 |
|                    | 6.  El Cajero entrega el dinero y registra el   |
|                    |     > desembolso en el Libro de Ingresos y      |
|                    |     > Egresos.                                  |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si alguna validación falla (monto,          |
| alternativo:**     |     > relación, garantía, fiadores cruzados),   |
|                    |     > el sistema bloquea el desembolso hasta    |
|                    |     > corregir.                                 |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El crédito queda desembolsado, activo y con su  |
|                    | tabla de pagos calculada.                       |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-13 --- Registrar Pago de Cuota**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-13                                           |
+====================+=================================================+
| **Nombre:**        | Registrar Pago de Cuota                         |
+--------------------+-------------------------------------------------+
| **Actor            | Cajero, Contable, Actualizador                  |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un crédito vigente.                |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario selecciona el número de crédito  |
| principal:**       |     > y confirma los datos del socio.           |
|                    |                                                 |
|                    | 2.  El sistema calcula el interés de la cuota   |
|                    |     > sobre saldo decreciente y el capital fijo |
|                    |     > correspondiente.                          |
|                    |                                                 |
|                    | 3.  El sistema valida que el monto pagado no    |
|                    |     > supere el saldo pendiente.                |
|                    |                                                 |
|                    | 4.  El Cajero recibe el pago y el sistema lo    |
|                    |     > registra, actualizando el saldo del       |
|                    |     > crédito.                                  |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si la cuota se paga después de su fecha de  |
| alternativo:**     |     > vencimiento, el sistema lo señala como    |
|                    |     > atraso.                                   |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El pago queda registrado y el saldo del crédito |
|                    | actualizado.                                    |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-14 --- Registrar Pago de Deuda Pendiente (PDP)**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-14                                           |
+====================+=================================================+
| **Nombre:**        | Registrar Pago de Deuda Pendiente (PDP)         |
+--------------------+-------------------------------------------------+
| **Actor            | Contable                                        |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un socio retirado con deuda        |
|                    | pendiente en Incobrables.                       |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario busca el crédito del socio       |
| principal:**       |     > retirado.                                 |
|                    |                                                 |
|                    | 2.  El socio realiza un abono o el pago total   |
|                    |     > de su deuda.                              |
|                    |                                                 |
|                    | 3.  El sistema registra el pago y lo refleja    |
|                    |     > como \"Otros Ingresos\" tras el Cierre    |
|                    |     > del mes.                                  |
|                    |                                                 |
|                    | 4.  Si el pago cancela toda la deuda, el        |
|                    |     > sistema actualiza el estatus del socio a  |
|                    |     > \"Retirado con Deuda Pagada\".            |
+--------------------+-------------------------------------------------+
| **Flujo            | -   El pago debe reponer el monto               |
| alternativo:**     |     > correspondiente en la Reserva por         |
|                    |     > Incobrables.                              |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La deuda del socio retirado queda actualizada   |
|                    | (parcial o totalmente saldada).                 |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-15 --- Refinanciar Crédito**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-15                                           |
+====================+=================================================+
| **Nombre:**        | Refinanciar Crédito                             |
+--------------------+-------------------------------------------------+
| **Actor            | Verificador, Cajero, Contable                   |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un crédito vigente.                |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario selecciona el crédito a          |
| principal:**       |     > refinanciar; el sistema muestra el saldo  |
|                    |     > de capital pendiente.                     |
|                    |                                                 |
|                    | 2.  El usuario decide si los intereses          |
|                    |     > pendientes se pagan en efectivo o se      |
|                    |     > incluyen en el nuevo crédito.             |
|                    |                                                 |
|                    | 3.  El sistema registra el cierre contable del  |
|                    |     > crédito anterior.                         |
|                    |                                                 |
|                    | 4.  El sistema abre un nuevo crédito precargado |
|                    |     > con los datos del anterior, permitiendo   |
|                    |     > modificar monto, tasas, fiadores y fecha  |
|                    |     > de vencimiento.                           |
|                    |                                                 |
|                    | 5.  Si se incluyeron intereses pendientes, el   |
|                    |     > sistema los suma automáticamente al nuevo |
|                    |     > monto.                                    |
|                    |                                                 |
|                    | 6.  El sistema calcula la nueva tabla de pagos  |
|                    |     > con el modelo de saldo decreciente.       |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el socio decide pagar los intereses      |
| alternativo:**     |     > pendientes en efectivo, el nuevo crédito  |
|                    |     > se abre solo por el capital.              |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El crédito anterior queda cerrado contablemente |
|                    | y uno nuevo queda activo con las condiciones    |
|                    | actualizadas.                                   |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-16 --- Registrar Otros Ingresos / Otros Egresos**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-16                                           |
+====================+=================================================+
| **Nombre:**        | Registrar Otros Ingresos / Otros Egresos        |
+--------------------+-------------------------------------------------+
| **Actor            | Cajero                                          |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El usuario debe estar ubicado en un Bankomunal. |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario selecciona la operación (Otro    |
| principal:**       |     > Ingreso u Otro Egreso).                   |
|                    |                                                 |
|                    | 2.  El usuario ingresa fecha y monto.           |
|                    |                                                 |
|                    | 3.  El sistema registra la operación en el      |
|                    |     > Libro de Ingresos y Egresos.              |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el monto proviene de una Donación, el    |
| alternativo:**     |     > sistema lo registra automáticamente como  |
|                    |     > Ingreso al Fondo para Gastos, en vez de   |
|                    |     > Otro Ingreso.                             |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La operación queda registrada en el Libro de    |
|                    | Ingresos y Egresos.                             |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-17 --- Gestionar Fondo de Gastos**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-17                                           |
+====================+=================================================+
| **Nombre:**        | Gestionar Fondo de Gastos                       |
+--------------------+-------------------------------------------------+
| **Actor            | Contable                                        |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El Fondo para Gastos debe estar configurado     |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario selecciona la operación (Ingreso |
| principal:**       |     > al Fondo o Gasto del Bankomunal).         |
|                    |                                                 |
|                    | 2.  El usuario ingresa fecha y monto.           |
|                    |                                                 |
|                    | 3.  El sistema actualiza el saldo acumulado del |
|                    |     > Fondo para Gastos.                        |
+--------------------+-------------------------------------------------+
| **Flujo            | Si se registra un Bien Adquirido, el sistema lo |
| alternativo:**     | contabiliza como Activo Fijo sin afectar el     |
|                    | saldo de caja.                                  |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El saldo del Fondo para Gastos queda            |
|                    | actualizado.                                    |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-18 --- Corregir una Operación**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-18                                           |
|====================+=================================================+
| **Nombre:**        | Corregir una Operación                          |
+--------------------+-------------------------------------------------+
| **Actor            | Contable                                        |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir una operación previamente          |
|                    | registrada en el Libro de Ingresos y Egresos.   |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario ubica la operación a corregir.   |
| principal:**       |                                                 |
|                    | 2.  El sistema verifica si el mes ya fue        |
|                    |     > cerrado.                                  |
|                    |                                                 |
|                    | 3.  El usuario realiza la corrección (monto,    |
|                    |     > fecha u otro dato).                       |
|                    |                                                 |
|                    | 4.  El sistema guarda la corrección.            |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si la corrección se hace después del Cierre |
| alternativo:**     |     > de Mes, el sistema solicita el nombre de  |
|                    |     > quien corrige y registra el cambio en     |
|                    |     > Auditoría (no solo en el Libro de I/E).   |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La operación queda corregida, con registro de   |
|                    | auditoría si aplicó.                            |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-19 --- Proceso de Cuadre**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-19                                           |
+====================+=================================================+
| **Nombre:**        | Ejecutar Proceso de Cuadre                      |
+--------------------+-------------------------------------------------+
| **Actor            | Contable                                        |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Deben existir operaciones registradas en el mes |
|                    | a cuadrar.                                      |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario ejecuta el Proceso de Cuadre.    |
| principal:**       |                                                 |
|                    | 2.  El sistema genera el Informe de Gestión     |
|                    |     > Mensual y el Balance del Mes.             |
|                    |                                                 |
|                    | 3.  El usuario compara la Disponibilidad de     |
|                    |     > Efectivo calculada contra el efectivo     |
|                    |     > real en caja.                             |
|                    |                                                 |
|                    | 4.  Si coincide, el usuario continúa al Cierre  |
|                    |     > de Mes (CU-20).                           |
+--------------------+-------------------------------------------------+
| **Flujo            | Si el Balance no cuadra, el sistema alerta y el |
| alternativo:**     | usuario debe corregir operaciones y repetir el  |
|                    | Cuadre las veces necesarias.                    |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El mes queda \"cuadrado\" y listo para su       |
|                    | cierre definitivo, o pendiente de corrección.   |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-20 --- Cierre de Mes**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-20                                           |
+====================+=================================================+
| **Nombre:**        | Cierre de Mes                                   |
+--------------------+-------------------------------------------------+
| **Actor            | Contable                                        |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | El Proceso de Cuadre debe haberse ejecutado     |
|                    | exitosamente; el mes anterior debe estar        |
|                    | cerrado.                                        |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario ejecuta el Cierre Mensual.       |
| principal:**       |                                                 |
|                    | 2.  El sistema calcula y actualiza los saldos   |
|                    |     > del Fondo de Gastos y del Fondo de        |
|                    |     > Incobrables.                              |
|                    |                                                 |
|                    | 3.  El sistema recalcula el PPCFC incluyendo el |
|                    |     > mes cerrado.                              |
|                    |                                                 |
|                    | 4.  El sistema sella el mes como cerrado.       |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si es necesario corregir algo después del   |
| alternativo:**     |     > cierre, el usuario puede hacerlo          |
|                    |     > mediante, quedando registrada la          |
|                    |     > auditoría.                                |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El mes queda cerrado definitivamente, con sus   |
|                    | fondos y PPCFC actualizados.                    |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-21 --- Generar Reportes**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-21                                           |
+====================+=================================================+
| **Nombre:**        | Generar Reportes                                |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Deben existir datos registrados en el sistema.  |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra al módulo de Reportes.     |
| principal:**       |                                                 |
|                    | 2.  El usuario selecciona el reporte deseado    |
|                    |     > (Socios Activos, Gestión, Balance         |
|                    |     > General, Fiadores, Créditos               |
|                    |     > Vencidos/Vigentes, Libro de I/E, Control  |
|                    |     > de Acciones, Rendimiento de Acciones).    |
|                    |                                                 |
|                    | 3.  El sistema genera el reporte con la         |
|                    |     > información solicitada.                   |
|                    |                                                 |
|                    | 4.  El usuario exporta o imprime el reporte si  |
|                    |     > lo requiere.                              |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el reporte requiere un rango de fechas   |
| alternativo:**     |     > (ej. Rendimiento de Acciones), el sistema |
|                    |     > solicita esos parámetros antes de         |
|                    |     > generarlo.                                |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El reporte queda generado y disponible para el  |
|                    | usuario.                                        |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-22 --- Respaldar Información**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-22                                           |
+====================+=================================================+
| **Nombre:**        | Respaldar Información                           |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un Bankomunal con información      |
|                    | registrada.                                     |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra a la opción de Respaldo.   |
| principal:**       |                                                 |
|                    | 2.  El usuario elige la ubicación donde guardar |
|                    |     > el archivo.                               |
|                    |                                                 |
|                    | 3.  El sistema genera el archivo de respaldo    |
|                    |     > con toda la información del Bankomunal.   |
|                    |                                                 |
|                    | 4.  El sistema registra quién hizo el respaldo  |
|                    |     > y cuándo.                                 |
+--------------------+-------------------------------------------------+
| **Flujo            |                                                 |
| alternativo:**     |                                                 |
+--------------------+-------------------------------------------------+
| **Postcondición:** | Queda un archivo de respaldo disponible para    |
|                    | restaurar en caso de pérdida de información o   |
|                    | cambio de equipo.                               |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-23 --- Restaurar Información**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-23                                           |
+====================+=================================================+
| **Nombre:**        | Restaurar Información                           |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir un archivo de respaldo válido      |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra a la opción de Restaurar.  |
| principal:**       |                                                 |
|                    | 2.  El usuario selecciona el archivo de         |
|                    |     > respaldo.                                 |
|                    |                                                 |
|                    | 3.  El sistema alerta que esta acción puede     |
|                    |     > sobrescribir la información actual.       |
|                    |                                                 |
|                    | 4.  El usuario confirma.                        |
|                    |                                                 |
|                    | 5.  El sistema valida que el archivo            |
|                    |     > corresponda al Bankomunal correcto y      |
|                    |     > restaura la información.                  |
|                    |                                                 |
|                    | 6.  El sistema solicita el nombre de quien      |
|                    |     > realiza el cambio y registra la           |
|                    |     > modificación en Auditoría.                |
|                    |                                                 |
|                    | 7.  El sistema registra quién hizo la           |
|                    |     > restauración y cuándo.                    |
+--------------------+-------------------------------------------------+
| **Flujo            | -   Si el archivo no corresponde al Bankomunal  |
| alternativo:**     |     > correcto, el sistema bloquea la           |
|                    |     > restauración y muestra un error.          |
+--------------------+-------------------------------------------------+
| **Postcondición:** | La información del Bankomunal queda restaurada  |
|                    | según el archivo de respaldo seleccionado.      |
+--------------------+-------------------------------------------------+

**Caso de Uso: CU-24 --- Consultar Auditoría**

+--------------------+-------------------------------------------------+
| **ID:**            | CU-24                                           |
+====================+=================================================+
| **Nombre:**        | Consultar Auditoría                             |
+--------------------+-------------------------------------------------+
| **Actor            | Todos                                           |
| principal:**       |                                                 |
+--------------------+-------------------------------------------------+
| **Precondición:**  | Debe existir al menos un registro de auditoría. |
+--------------------+-------------------------------------------------+
| **Flujo            | 1.  El usuario entra al módulo de Auditoría.    |
| principal:**       |                                                 |
|                    | 2.  El sistema muestra la bitácora completa     |
|                    |     > (fecha, quién, qué cambió, valor          |
|                    |     > anterior/nuevo, tipo).                    |
|                    |                                                 |
|                    | 3.  El usuario filtra por fecha o tipo de       |
|                    |     > cambio si lo necesita.                    |
+--------------------+-------------------------------------------------+
| **Flujo            |                                                 |
| alternativo:**     |                                                 |
+--------------------+-------------------------------------------------+
| **Postcondición:** | El usuario visualiza el historial de cambios    |
|                    | sensibles del sistema.                          |
+--------------------+-------------------------------------------------+

# **5. Épicas del Proyecto**

  ------------------------------------------------------------------------
  **ID**      **Nombre de la Épica** **Descripción**
  ----------- ---------------------- -------------------------------------
  **EP-01**   Autenticación y        Todo lo relacionado con el ingreso al
              Usuarios               sistema, la contraseña genérica, la
                                     creación y selección de Bankomunales,
                                     y el aislamiento de datos entre
                                     ellos.

  **EP-02**   Configuración del      Todo lo relacionado con los Datos
              Bankomunal             Generales del Bankomunal y sus
                                     condiciones configurables: tasas de
                                     crédito, % de garantía, % de
                                     retención de fondos y valor nominal
                                     de la acción.

  **EP-03**   Gestión de Socios      Todo lo relacionado con el registro,
                                     consulta y actualización de los
                                     socios, sus datos personales,
                                     beneficiarios y protegidos.

  **EP-04**   Gestión de Acciones    Todo lo relacionado con la compra,
                                     liquidación (parcial y total) y
                                     reparto de ganancias de las acciones
                                     de los socios.

  **EP-05**   Gestión de Créditos    Todo lo relacionado con la solicitud,
                                     aprobación, desembolso, pago de
                                     cuotas, pago de deuda pendiente y
                                     refinanciamiento de los créditos
                                     otorgados a los socios.

  **EP-06**   Caja y Contabilidad    Todo lo relacionado con el registro
                                     de otros ingresos/egresos, el manejo
                                     del Fondo para Gastos, y la
                                     corrección de operaciones en el Libro
                                     de Ingresos y Egresos.

  **EP-07**   Cierre Contable        Todo lo relacionado con el Proceso de
              Mensual                Cuadre y el Cierre de Mes, incluyendo
                                     la actualización de fondos y el
                                     recálculo del PPCFC.

  **EP-08**   Reportes y Consultas   Todo lo relacionado con la
                                     generación, exportación e impresión
                                     de los reportes administrativos y
                                     financieros del Bankomunal.

  **EP-09**   Respaldo y Continuidad Todo lo relacionado con la generación
              de Datos               y restauración de respaldos de
                                     información, para proteger los datos
                                     del Bankomunal ante pérdida o cambio
                                     de equipo.

  **EP-10**   Auditoría y            Todo lo relacionado con el registro y
              Trazabilidad           consulta de cambios sensibles del
                                     sistema (configuración, correcciones,
                                     respaldos), sin implicar control de
                                     acceso por rol.
  ------------------------------------------------------------------------

# **6. Historias de usuario** 

**Historia HU-01**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-01*                                           |
+==================+===================================================+
| **ID**           | HU-01                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como usuario de la junta o socio, quiero ingresar |
|                  | al sistema con una contraseña genérica, para      |
|                  | poder registrar y consultar operaciones del       |
|                  | Bankomunal.                                       |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Ingreso válido con la contraseña genérica*     |
| aceptación**     |                                                   |
|                  | *• Mensaje de error si es incorrecta, sin límite  |
|                  | de reintentos*                                    |
|                  |                                                   |
|                  | *• Si hay varios Bankomunales, se muestra         |
|                  | selección; si hay uno solo, entra directo*        |
|                  |                                                   |
|                  | *• Puede volver a la pantalla de selección sin    |
|                  | cerrar la app*                                    |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-02**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-01*                                           |
+==================+===================================================+
| **ID**           | HU-02                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Administrador, quiero crear un nuevo         |
|                  | Bankomunal la primera vez que uso el sistema,     |
|                  | para empezar a registrar su información.          |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Solicita clave de configuración inicial*       |
| aceptación**     |                                                   |
|                  | *• Valida que el nombre/código no esté duplicado* |
|                  |                                                   |
|                  | *• Confirma la creación exitosa*                  |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-03**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-01*                                           |
+==================+===================================================+
| **ID**           | HU-03                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como usuario, quiero seleccionar con cuál         |
|                  | Bankomunal trabajar cuando hay varios en el mismo |
|                  | computador, para que la información de cada uno   |
|                  | no se mezcle.                                     |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Lista de Bankomunales visible si hay más de    |
| aceptación**     | uno*                                              |
|                  |                                                   |
|                  | *• Entrada directa si hay solo uno*               |
|                  |                                                   |
|                  | *• Los datos de cada Bankomunal quedan aislados   |
|                  | entre sí*                                         |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-04**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-02*                                           |
+==================+===================================================+
| **ID**           | HU-04                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Administrador, quiero configurar y consultar |
|                  | las condiciones del Bankomunal (tasas, garantías, |
|                  | % de fondos, valor de la acción), para que el     |
|                  | sistema aplique correctamente las reglas de       |
|                  | negocio.                                          |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Datos generales visibles (solo consulta)*      |
| aceptación**     |                                                   |
|                  | *• Créditos, acciones vendidas y saldos de fondos |
|                  | se calculan automáticamente*                      |
|                  |                                                   |
|                  | *• Condiciones de crédito, % de garantía, % de    |
|                  | fondos y valor nominal son editables*             |
|                  |                                                   |
|                  | *• Confirmación al guardar*                       |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-05**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-03*                                           |
+==================+===================================================+
| **ID**           | HU-05                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Actualizador, quiero registrar un nuevo      |
|                  | socio con sus datos personales y beneficiarios,   |
|                  | para llevar su información completa desde el      |
|                  | inicio.                                           |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Registro desde pantalla independiente o        |
| aceptación**     | automático al comprar acciones*                   |
|                  |                                                   |
|                  | *• Captura datos personales, beneficiario y       |
|                  | protegidos*                                       |
|                  |                                                   |
|                  | *• Valida cédula no duplicada*                    |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-06**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-03*                                           |
+==================+===================================================+
| **ID**           | HU-06                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Actualizador, quiero consultar y actualizar  |
|                  | los datos de un socio en cualquier momento, para  |
|                  | mantener su información al día.                   |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Búsqueda por nombre o cédula*                  |
| aceptación**     |                                                   |
|                  | *• Edición y guardado de cambios*                 |
|                  |                                                   |
|                  | *• Consulta sin modificar si no hay cambios*      |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-07**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-04*                                           |
+==================+===================================================+
| **ID**           | HU-07                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como socio, quiero comprar acciones del           |
|                  | Bankomunal, para tener derecho a créditos y a las |
|                  | ganancias que generen.                            |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Cálculo automático de la cantidad de acciones  |
| aceptación**     | según el valor nominal*                           |
|                  |                                                   |
|                  | *• Cálculo del % de participación*                |
|                  |                                                   |
|                  | *• Bloqueo si supera el 15% (RN-02)*              |
|                  |                                                   |
|                  | *• Validación del PPCFC antes de autorizar la     |
|                  | venta*                                            |
|                  |                                                   |
|                  | *• Registro del mes de compra*                    |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-08**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-04*                                           |
+==================+===================================================+
| **ID**           | HU-08                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como socio, quiero liquidar mis acciones de forma |
|                  | parcial o total, para recuperar mi inversión y    |
|                  | ganancias cuando lo necesite.                     |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Cálculo de acciones libres (descontando        |
| aceptación**     | garantías)*                                       |
|                  |                                                   |
|                  | *• Bloqueo si están comprometidas en un crédito   |
|                  | vigente*                                          |
|                  |                                                   |
|                  | *• Los 3 escenarios de deuda (sin deuda / menor / |
|                  | mayor) se calculan correctamente*                 |
|                  |                                                   |
|                  | *• Traslado a Incobrables si aplica*              |
|                  |                                                   |
|                  | *• Actualización de estatus si es liquidación     |
|                  | total*                                            |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-09**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-04*                                           |
+==================+===================================================+
| **ID**           | HU-09                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como socio, quiero recibir las ganancias          |
|                  | generadas por mis acciones, para obtener un       |
|                  | retorno de mi inversión en el Bankomunal.         |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Identificación de acciones que cumplen su año* |
| aceptación**     |                                                   |
|                  | *• Cálculo del Balance Neto repartible*           |
|                  |                                                   |
|                  | *• Cálculo del valor de ganancia por acción*      |
|                  |                                                   |
|                  | *• Registro del pago a cada socio*                |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-10**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-05*                                           |
+==================+===================================================+
| **ID**           | HU-10                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como socio, quiero solicitar un crédito indicando |
|                  | mis ingresos, egresos y fiadores, para que la     |
|                  | Junta pueda evaluarlo y aprobarlo.                |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Captura de datos del solicitante,              |
| aceptación**     | ingresos/egresos y destino*                       |
|                  |                                                   |
|                  | *• Cálculo de capacidad de pago y tabla de pagos* |
|                  |                                                   |
|                  | *• Registro de hasta 2 fiadores con sus           |
|                  | garantías*                                        |
|                  |                                                   |
|                  | *• Validación de la garantía mínima*              |
|                  |                                                   |
|                  | *• Registro de decisión:                          |
|                  | Aprobado/Modificado/Negado/Diferido*              |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-11**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-05*                                           |
+==================+===================================================+
| **ID**           | HU-11                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Junta, quiero desembolsar un crédito ya      |
|                  | aprobado, para entregar el dinero al socio bajo   |
|                  | las condiciones validadas.                        |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Asignación automática del número de crédito*   |
| aceptación**     |                                                   |
|                  | *• Validación de monto máximo, relación 1 a 5,    |
|                  | garantía 40% y fiadores cruzados*                 |
|                  |                                                   |
|                  | *• Cálculo de la tabla de pagos con saldo         |
|                  | decreciente*                                      |
|                  |                                                   |
|                  | *• Registro en el Libro de Ingresos y Egresos*    |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-12**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-05*                                           |
+==================+===================================================+
| **ID**           | HU-12                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Cajero, quiero registrar el pago de una      |
|                  | cuota de crédito, para actualizar el saldo del    |
|                  | socio de forma correcta.                          |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Cálculo automático del interés sobre saldo     |
| aceptación**     | decreciente*                                      |
|                  |                                                   |
|                  | *• Cálculo automático del capital fijo por cuota* |
|                  |                                                   |
|                  | *• Validación de que el pago no exceda el saldo   |
|                  | pendiente*                                        |
|                  |                                                   |
|                  | *• Actualización automática del saldo del         |
|                  | crédito*                                          |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-13**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-05*                                           |
+==================+===================================================+
| **ID**           | HU-13                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Contable, quiero registrar los pagos de un   |
|                  | socio retirado con deuda pendiente, para que      |
|                  | pueda saldar lo que debe al Bankomunal.           |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Búsqueda del crédito en Incobrables*           |
| aceptación**     |                                                   |
|                  | *• Registro de uno o varios abonos*               |
|                  |                                                   |
|                  | *• Reflejo como \"Otros Ingresos\" tras el        |
|                  | cierre*                                           |
|                  |                                                   |
|                  | *• Cambio de estatus si se cancela toda la deuda* |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-14**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-05*                                           |
+==================+===================================================+
| **ID**           | HU-14                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como socio, quiero refinanciar mi crédito         |
|                  | vigente, para obtener más tiempo o mejores        |
|                  | condiciones de pago.                              |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Muestra el saldo pendiente del crédito*        |
| aceptación**     |                                                   |
|                  | *• Permite decidir si los intereses se pagan en   |
|                  | efectivo o se incluyen*                           |
|                  |                                                   |
|                  | *• Cierra el crédito anterior y abre uno nuevo    |
|                  | precargado*                                       |
|                  |                                                   |
|                  | *• Calcula la nueva tabla de pagos*               |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-15**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-06*                                           |
+==================+===================================================+
| **ID**           | HU-15                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Cajero, quiero registrar otros ingresos y    |
|                  | egresos que no correspondan a acciones o          |
|                  | créditos, para mantener la caja del Bankomunal    |
|                  | completa y actualizada.                           |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Registro de fecha y monto*                     |
| aceptación**     |                                                   |
|                  | *• Las donaciones se registran automáticamente    |
|                  | como ingreso al Fondo para Gastos*                |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-16**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-06*                                           |
+==================+===================================================+
| **ID**           | HU-16                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Cajero, quiero gestionar los ingresos y      |
|                  | gastos del Fondo para Gastos del Bankomunal, para |
|                  | cubrir sus necesidades operativas.                |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Actualización del saldo acumulado del fondo*   |
| aceptación**     |                                                   |
|                  | *• Registro de bienes adquiridos como activo fijo |
|                  | sin afectar la caja*                              |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-17**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-06*                                           |
+==================+===================================================+
| **ID**           | HU-17                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Contable, quiero corregir una operación      |
|                  | registrada por error, para mantener la            |
|                  | información del Libro de Ingresos y Egresos       |
|                  | correcta.                                         |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Corrección permitida antes del cierre de mes*  |
| aceptación**     |                                                   |
|                  | *• Corrección permitida después del cierre, con   |
|                  | registro de auditoría (quién, cuándo, motivo)*    |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-18**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-07*                                           |
+==================+===================================================+
| **ID**           | HU-18                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Contable, quiero ejecutar el Proceso de      |
|                  | Cuadre del mes, para verificar que la información |
|                  | esté correcta antes de cerrarlo.                  |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Puede repetirse sin afectar datos definitivos* |
| aceptación**     |                                                   |
|                  | *• Genera Informe de Gestión y Balance del Mes*   |
|                  |                                                   |
|                  | *• Alerta si el balance no cuadra*                |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-19**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-07*                                           |
+==================+===================================================+
| **ID**           | HU-19                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Contable, quiero ejecutar el Cierre Mensual  |
|                  | definitivo, para sellar la información del mes y  |
|                  | actualizar los fondos del Bankomunal.             |
+------------------+---------------------------------------------------+
| **Criterios de   | *• No permite cerrar si el mes anterior sigue     |
| aceptación**     | abierto*                                          |
|                  |                                                   |
|                  | *• Actualiza automáticamente los fondos de Gastos |
|                  | e Incobrables*                                    |
|                  |                                                   |
|                  | *• Recalcula el PPCFC*                            |
|                  |                                                   |
|                  | *• Permite correcciones posteriores con           |
|                  | auditoría*                                        |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-20**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-08*                                           |
+==================+===================================================+
| **ID**           | HU-20                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como usuario de la Junta, quiero generar los      |
|                  | distintos reportes del Bankomunal, para consultar |
|                  | y presentar la información administrativa y       |
|                  | financiera.                                       |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Genera los 9 reportes definidos (Socios        |
| aceptación**     | Activos, Gestión, Balance General, Fiadores,      |
|                  | Créditos Vencidos/Vigentes, Libro de I/E, Control |
|                  | de Acciones, Rendimiento de Acciones)*            |
|                  |                                                   |
|                  | *• Permite exportar/imprimir cualquiera de ellos* |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-21**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-09*                                           |
+==================+===================================================+
| **ID**           | HU-21                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Administrador, quiero generar un respaldo de |
|                  | la información del Bankomunal, para protegerla    |
|                  | ante fallas del computador o pérdida de datos.    |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Genera el archivo en la ubicación elegida*     |
| aceptación**     |                                                   |
|                  | *• Registra quién hizo el respaldo y cuándo*      |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-22**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-09*                                           |
+==================+===================================================+
| **ID**           | HU-22                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Administrador, quiero restaurar un respaldo  |
|                  | previamente generado, para recuperar la           |
|                  | información del Bankomunal si algo falla.         |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Alerta antes de sobrescribir datos actuales*   |
| aceptación**     |                                                   |
|                  | *• Valida que el archivo corresponda al           |
|                  | Bankomunal correcto*                              |
|                  |                                                   |
|                  | *• Registra quién restauró y cuándo*              |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+

**Historia HU-23**

+------------------+---------------------------------------------------+
| **Épica**        | *EP-010*                                          |
+==================+===================================================+
| **ID**           | HU-23                                             |
+------------------+---------------------------------------------------+
| **Historia**     | Como Administrador, quiero consultar el historial |
|                  | de cambios sensibles del sistema, para poder      |
|                  | verificar quién modificó la configuración o       |
|                  | corrigió una operación.                           |
+------------------+---------------------------------------------------+
| **Criterios de   | *• Configuración con valor anterior/nuevo*        |
| aceptación**     |                                                   |
|                  | *• Solicita nombre antes de confirmar cambios     |
|                  | sensibles*                                        |
|                  |                                                   |
|                  | *• Permite filtrar el historial por fecha o       |
|                  | tipo.*                                            |
+------------------+---------------------------------------------------+
| **Estado**       | Pendiente                                         |
+------------------+---------------------------------------------------+
