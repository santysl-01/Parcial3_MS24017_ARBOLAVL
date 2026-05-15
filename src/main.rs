/*
    =======================================================================
    INFORME TÉCNICO FASE 1: PROPIEDAD, MEMORIA Y PRUEBA DE ESCRITORIO
    =======================================================================

    [A] Análisis de Option::take() y Ownership:
    El compilador de Rust restringe múltiples accesos mutables para evitar 
    fugas de memoria y condiciones de carrera. Durante las rotaciones AVL, 
    no podemos simplemente reasignar punteros de las ramas. Al utilizar `.take()`, 
    reemplazamos de forma segura el valor del nodo por `None` in-place, 
    adquiriendo la propiedad temporal de la rama para reubicarla sin hacer 
    clonaciones de memoria (Zero-Cost Abstraction).

    [B] ¿Por qué utilizar Box<NodoAvl>?
    Debido a que un árbol es una estructura recursiva (Nodos dentro de Nodos), 
    su tamaño teórico sería infinito en la pila (Stack). `Box` actúa como un 
    Smart Pointer: guarda los datos de la aeronave en el Heap (memoria dinámica) 
    y deja un puntero de tamaño fijo en el Stack, permitiendo que el compilador 
    conozca el tamaño exacto de la estructura.

    [C] Traza de Escritorio (Altitudes: 5000, 3000, 2000, 4000, 3500, 6000):
    1. 5000: Nodo raíz.
    2. 3000: Insertado a la izquierda. Balanceado.
    3. 2000: Insertado a la izquierda del 3000. 
       -> Alerta en 5000: Peso acumulado a la izquierda. Rotación simple a la derecha. 
       -> Nueva raíz: 3000.
    4. 4000: Insertado a la derecha de 3000, izquierda de 5000. Balanceado.
    5. 3500: Insertado a la izquierda de 4000. 
       -> Alerta en 5000: Peso a la izquierda. Rotación simple a la derecha sobre 5000.
    6. 6000: Insertado a la derecha de 5000.
       -> Alerta crítica en la raíz (3000): Peso acumulado hacia la derecha. 
       -> Rotación simple a la izquierda sobre el nodo 3000.
    => Topología Final: 
           4000
          /    \
      3000      5000
      /  \          \
   2000  3500       6000
*/

#[derive(Debug)]
struct Aeronave {
    identificador: String,
    altitud: u32,
}

struct NodoAvl {
    nave: Aeronave,
    rama_izq: Option<Box<NodoAvl>>,
    rama_der: Option<Box<NodoAvl>>,
    altura_nodo: i32,
}

impl NodoAvl {
    fn crear(nave: Aeronave) -> Self {
        NodoAvl {
            nave,
            rama_izq: None,
            rama_der: None,
            altura_nodo: 1,
        }
    }
}

// ==========================================================
// MÓDULO DE EQUILIBRIO ESTÁTICO Y ROTACIONES
// ==========================================================

fn obtener_altura(nodo: &Option<Box<NodoAvl>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura_nodo)
}

fn actualizar_altura(nodo: &mut NodoAvl) {
    nodo.altura_nodo = 1 + std::cmp::max(
        obtener_altura(&nodo.rama_izq),
        obtener_altura(&nodo.rama_der),
    );
}

fn evaluar_balance(nodo: &NodoAvl) -> i32 {
    obtener_altura(&nodo.rama_izq) - obtener_altura(&nodo.rama_der)
}

fn rotacion_derecha(mut y: Box<NodoAvl>) -> Box<NodoAvl> {
    let mut x = y.rama_izq.take().unwrap();
    y.rama_izq = x.rama_der.take();
    actualizar_altura(&mut y);
    x.rama_der = Some(y);
    actualizar_altura(&mut x);
    x
}

fn rotacion_izquierda(mut x: Box<NodoAvl>) -> Box<NodoAvl> {
    let mut y = x.rama_der.take().unwrap();
    x.rama_der = y.rama_izq.take();
    actualizar_altura(&mut x);
    y.rama_izq = Some(x);
    actualizar_altura(&mut y);
    y
}

fn rebalancear(mut arbol: Box<NodoAvl>) -> Box<NodoAvl> {
    actualizar_altura(&mut arbol);
    let balance = evaluar_balance(&arbol);

    if balance > 1 {
        if evaluar_balance(arbol.rama_izq.as_ref().unwrap()) < 0 {
            let izq = arbol.rama_izq.take().unwrap();
            arbol.rama_izq = Some(rotacion_izquierda(izq));
        }
        return rotacion_derecha(arbol);
    }
    if balance < -1 {
        if evaluar_balance(arbol.rama_der.as_ref().unwrap()) > 0 {
            let der = arbol.rama_der.take().unwrap();
            arbol.rama_der = Some(rotacion_derecha(der));
        }
        return rotacion_izquierda(arbol);
    }
    arbol
}

// ==========================================================
// REGISTRO DE AERONAVES (INSERCIÓN)
// ==========================================================

fn agregar_vuelo(raiz_opt: Option<Box<NodoAvl>>, nueva_nave: Aeronave) -> Box<NodoAvl> {
    let mut raiz = match raiz_opt {
        None => return Box::new(NodoAvl::crear(nueva_nave)),
        Some(nodo) => nodo,
    };

    if nueva_nave.altitud < raiz.nave.altitud {
        raiz.rama_izq = Some(agregar_vuelo(raiz.rama_izq.take(), nueva_nave));
    } else if nueva_nave.altitud > raiz.nave.altitud {
        raiz.rama_der = Some(agregar_vuelo(raiz.rama_der.take(), nueva_nave));
    } else {
        return raiz; 
    }

    rebalancear(raiz)
}

// ==========================================================
// MÓDULO DE RASTREO Y PROXIMIDAD
// ==========================================================

fn buscar_identificador(raiz: &Option<Box<NodoAvl>>, altitud_buscada: u32) -> Option<&Aeronave> {
    let mut actual = raiz;
    while let Some(nodo) = actual {
        if altitud_buscada < nodo.nave.altitud {
            actual = &nodo.rama_izq;
        } else if altitud_buscada > nodo.nave.altitud {
            actual = &nodo.rama_der;
        } else {
            return Some(&nodo.nave);
        }
    }
    None
}

fn contar_vuelos_rango(raiz: &Option<Box<NodoAvl>>, minimo: u32, maximo: u32) -> usize {
    if let Some(nodo) = raiz {
        let mut contador = 0;
        if nodo.nave.altitud >= minimo && nodo.nave.altitud <= maximo {
            contador += 1;
        }
        if minimo < nodo.nave.altitud {
            contador += contar_vuelos_rango(&nodo.rama_izq, minimo, maximo);
        }
        if maximo > nodo.nave.altitud {
            contador += contar_vuelos_rango(&nodo.rama_der, minimo, maximo);
        }
        contador
    } else {
        0
    }
}

// ==========================================================
// DESCENSO DE VUELOS (ELIMINACIÓN IN-ORDER)
// ==========================================================

fn extraer_predecesor(mut nodo: Box<NodoAvl>) -> (Option<Box<NodoAvl>>, Aeronave) {
    if nodo.rama_der.is_none() {
        return (nodo.rama_izq.take(), nodo.nave);
    }
    let sub_der = nodo.rama_der.take().unwrap();
    let (nuevo_der, nave_max) = extraer_predecesor(sub_der);
    nodo.rama_der = nuevo_der;
    (Some(rebalancear(nodo)), nave_max)
}

fn eliminar_por_altitud(raiz_opt: Option<Box<NodoAvl>>, altitud_baja: u32) -> Option<Box<NodoAvl>> {
    let mut raiz = match raiz_opt {
        None => return None,
        Some(nodo) => nodo,
    };

    if altitud_baja < raiz.nave.altitud {
        raiz.rama_izq = eliminar_por_altitud(raiz.rama_izq.take(), altitud_baja);
    } else if altitud_baja > raiz.nave.altitud {
        raiz.rama_der = eliminar_por_altitud(raiz.rama_der.take(), altitud_baja);
    } else {
        if raiz.rama_izq.is_none() {
            return raiz.rama_der;
        } else if raiz.rama_der.is_none() {
            return raiz.rama_izq;
        }
        let rama_izq = raiz.rama_izq.take().unwrap();
        let (nueva_izq, nave_predecesora) = extraer_predecesor(rama_izq);
        raiz.nave = nave_predecesora;
        raiz.rama_izq = nueva_izq;
    }

    Some(rebalancear(raiz))
}

// ==========================================================
// VISUALIZACIÓN CLÁSICA ASCII DEL ÁRBOL
// ==========================================================

fn imprimir_in_order_limpio(raiz: &Option<Box<NodoAvl>>) {
    if let Some(nodo) = raiz {
        imprimir_in_order_limpio(&nodo.rama_izq);
        println!("Altitud: {:<5} | Vuelo: {}", nodo.nave.altitud, nodo.nave.identificador);
        imprimir_in_order_limpio(&nodo.rama_der);
    }
}

// Dibuja el árbol de forma lateral usando conectores ASCII puros
fn dibujar_arbol_ascii(nodo: &Option<Box<NodoAvl>>, prefijo: String, es_izquierdo: Option<bool>) {
    if let Some(n) = nodo {
        let mut prefijo_der = prefijo.clone();
        prefijo_der.push_str(match es_izquierdo {
            Some(true) => "│   ",
            Some(false) => "    ",
            None => "    ",
        });
        dibujar_arbol_ascii(&n.rama_der, prefijo_der, Some(false));

        let conector = match es_izquierdo {
            Some(true) => "└── ",
            Some(false) => "┌── ",
            None => "─── ",
        };
        println!("{}{}{} ({})", prefijo, conector, n.nave.altitud, n.nave.identificador);

        let mut prefijo_izq = prefijo.clone();
        prefijo_izq.push_str(match es_izquierdo {
            Some(true) => "    ",
            Some(false) => "│   ",
            None => "    ",
        });
        dibujar_arbol_ascii(&n.rama_izq, prefijo_izq, Some(true));
    }
}

// ==========================================================
// EJECUCIÓN PRINCIPAL Y PRUEBAS EN CONSOLA
// ==========================================================

fn main() {
    let mut sistema_avl: Option<Box<NodoAvl>> = None;

    let matriz_datos = vec![
        ("AV123", 5000), ("UA456", 3000), ("IB101", 2000),
        ("AF999", 4000), ("TA222", 3500), ("AM777", 6000),
    ];

    for (id, alt) in matriz_datos {
        sistema_avl = Some(agregar_vuelo(sistema_avl.take(), Aeronave {
            identificador: id.to_string(),
            altitud: alt,
        }));
    }

    println!("=== MOTOR DE GESTIÓN DE ESPACIO AÉREO (ÁRBOL AVL) ===\n");
    
    println!("--- FASE 1: Estado Inicial del Árbol (Topología ASCII) ---");
    println!("(Nota: Leer de izquierda a derecha. Arriba = Rama Derecha, Abajo = Rama Izquierda)\n");
    dibujar_arbol_ascii(&sistema_avl, "".to_string(), None);

    println!("\n--- FASE 1 B: Recorrido In-Order (Ascendente) ---");
    imprimir_in_order_limpio(&sistema_avl);

    println!("\n--- FASE 2: Búsqueda de Aeronave ---");
    let altitud_objetivo = 3500;
    print!("Buscando registro en {} pies... ", altitud_objetivo);
    match buscar_identificador(&sistema_avl, altitud_objetivo) {
        Some(nave) => println!("Encontrado. Identificador: {}", nave.identificador),
        None => println!("No se encontró ninguna aeronave en esa altitud."),
    }

    println!("\n--- FASE 4: Alerta de Proximidad ---");
    let limite_inferior = 3000;
    let limite_superior = 5000;
    let naves_en_rango = contar_vuelos_rango(&sistema_avl, limite_inferior, limite_superior);
    println!("Evaluando altitudes entre {} y {} pies...", limite_inferior, limite_superior);
    println!("Resultado: {} aeronaves detectadas en la zona de rango.", naves_en_rango);

    println!("\n--- FASE 3: Eliminación (Descenso y Aterrizaje) ---");
    let altitud_aterrizaje = 3000;
    println!("Iniciando secuencia de aterrizaje para la aeronave en {} pies...", altitud_aterrizaje);
    sistema_avl = eliminar_por_altitud(sistema_avl.take(), altitud_aterrizaje);
    println!("Aterrizaje completado. Rebalanceando el árbol...\n");

    println!("--- ESTADO FINAL DEL ÁRBOL AVL TRAS ELIMINACIÓN ---");
    dibujar_arbol_ascii(&sistema_avl, "".to_string(), None);
    println!("\nOperación concluida correctamente.");
}