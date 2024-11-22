# Space Travel Render

## Descripción

Space Travel Render es un simulador gráfico en 3D que permite explorar un sistema solar ficticio. Los usuarios pueden navegar por el espacio, observar diferentes cuerpos celestes y experimentar la iluminación dinámica proporcionada por el Sol.

- Video de ejecucion: https://youtu.be/eAouo8UbQ7k

## Requisitos

- Rust (versión 1.50 o superior)
- Cargo (incluido con Rust)

## Instalación

1. Clona el repositorio:

   ```bash
   git@github.com:DanielDubon/SpaceTravelRender.git
   ```

2. Asegúrate de tener las dependencias necesarias instaladas. Puedes usar el siguiente comando para instalar las dependencias:

   ```bash
   cargo build
   ```

## Ejecución

Para ejecutar el programa, utiliza el siguiente comando:

```bash
cargo run --release
```



Esto compilará el proyecto en modo de lanzamiento y ejecutará el simulador.

## Controles

- **W**: Avanzar hacia adelante.
- **S**: Retroceder.
- **A**: Girar a la izquierda.
- **D**: Girar a la derecha.
- **Up Arrow**: Inclinar hacia arriba.
- **Down Arrow**: Inclinar hacia abajo.
- **Q**: Mover hacia arriba.
- **E**: Mover hacia abajo.
- **B**: Cambiar a vista aérea (Bird Eye View).
- **1**: Teletransportar a la posición del Sol.
- **2**: Teletransportar a la posición de la Tierra.
- **3**: Teletransportar a la posición de Júpiter.
- **4**: Teletransportar a la posición de un agujero negro.
- **Esc**: Salir del programa.

## Modelos

Asegúrate de tener los modelos necesarios en la carpeta `assets/models/`:

- `esfera.obj`: Modelo de esfera.
- `nave.obj`: Modelo de la nave.
- `Rei_A-Pose_2.obj`: Modelo de Rei.

## Contribuciones

Las contribuciones son bienvenidas. Si deseas contribuir, por favor abre un issue o un pull request.

## Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.
