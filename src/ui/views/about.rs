//! About: what this is, what it watches, and what it is built on.

use std::collections::BTreeMap;

use rune::{El, ScrollSpec, col, el, row};
use solar_rust::domain::catalog;

use crate::ui::{
    State,
    components::{self, sun_mark},
    icons, theme,
};

use super::{Paint, page_header, tr};

pub fn view(state: &State, p: Paint) -> El {
    let t = p.t;
    let header = page_header(
        p,
        tr(state, "System / about", "Sistema / acerca de"),
        tr(state, "About", "Acerca de"),
        tr(
            state,
            "A native observatory for the Sun, built in Rust.",
            "Un observatorio nativo para el Sol, hecho en Rust.",
        ),
        vec![],
    );

    let hero = el()
        .key("about-hero")
        .w_full()
        .h(228.0)
        .padding(24.0)
        .radius(18.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .radial_gradient(
            [0.14, 0.2],
            [420.0, 320.0],
            [
                (0.0, t.amber.alpha(0.16)),
                (0.45, t.amber.alpha(0.04)),
                (1.0, rune::Color::TRANSPARENT),
            ],
        )
        .shadow(18.0, 46.0, rune::Color::BLACK.alpha(0.26))
        .child(
            row()
                .w_full()
                .h_full()
                .gap(22.0)
                .center_y()
                .child(sun_mark(96.0))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(8.0)
                        .child(components::micro(
                            tr(state, "Mission", "Misión"),
                            t.text_3,
                        ))
                        .child(components::label("Solar Observatory", theme::DISPLAY, t.text))
                        .child(
                            components::label(
                                tr(
                                    state,
                                    "Every channel from the Solar Dynamics Observatory, one archive, one click to the desktop. No account, no telemetry, no cloud.",
                                    "Todos los canales del Solar Dynamics Observatory, un archivo, un clic al escritorio. Sin cuenta, sin telemetría, sin nube.",
                                ),
                                theme::BODY,
                                t.text_2,
                            )
                            .wrap(),
                        )
                        .child(
                            row()
                                .gap(8.0)
                                .pt(4.0)
                                .center_y()
                                .child(components::chip(
                                    concat!("v", env!("CARGO_PKG_VERSION")),
                                    t.teal,
                                ))
                                .child(components::chip("Rune", t.violet))
                                .child(components::chip(
                                    tr(state, "Native", "Nativo"),
                                    t.amber,
                                )),
                        ),
                ),
        );

    let mut categories: BTreeMap<String, usize> = BTreeMap::new();
    for channel in catalog::channels() {
        *categories.entry(channel.category.clone()).or_insert(0) += 1;
    }

    let mission = col()
        .key("about-mission")
        .grow(1.0)
        .min_w(0.0)
        .padding(18.0)
        .gap(10.0)
        .radius(16.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(components::section_label(tr(
            state,
            "How it works",
            "Cómo funciona",
        )))
        .child(point(state, "01", tr(
            state,
            "A worker thread owns the archive and serializes every operation, so the interface never blocks on the network.",
            "Un hilo trabajador es dueño del archivo y serializa cada operación, así la interfaz nunca se bloquea por la red.",
        )))
        .child(point(state, "02", tr(
            state,
            "Images stay at their original resolution. Crops are stored as coordinates, never as rewritten files.",
            "Las imágenes se mantienen en su resolución original. Los recortes se guardan como coordenadas, nunca como archivos reescritos.",
        )))
        .child(point(state, "03", tr(
            state,
            "Retention is a rule, not a cleanup script: the archive keeps the newest captures and drops the rest.",
            "La retención es una regla, no un script de limpieza: el archivo conserva las capturas más nuevas y descarta el resto.",
        )));

    let instruments = col()
        .key("about-instruments")
        .w(340.0)
        .padding(18.0)
        .gap(9.0)
        .radius(16.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(components::section_label(tr(
            state,
            "Instruments",
            "Instrumentos",
        )))
        .child(components::label(
            tr(
                state,
                "Solar Dynamics Observatory · NASA",
                "Solar Dynamics Observatory · NASA",
            ),
            theme::BODY_SM,
            t.text_2,
        ))
        .children(
            categories
                .into_iter()
                .map(|(category, count)| {
                    row()
                        .key(format!("about-{category}"))
                        .w_full()
                        .h(30.0)
                        .center_y()
                        .justify_between()
                        .child(components::label(
                            category.to_string(),
                            theme::BODY,
                            t.text_2,
                        ))
                        .child(components::label(format!("{count}"), theme::MONO, t.text_3))
                })
                .collect::<Vec<_>>(),
        )
        .child(components::hairline())
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(rune::icon(&icons::GLOBE, 14.0).color(t.text_4))
                .child(components::label(
                    "sdo.gsfc.nasa.gov",
                    theme::MONO_SM,
                    t.text_3,
                )),
        );

    let credits = row()
        .key("about-credits")
        .w_full()
        .center_y()
        .gap(8.0)
        .child(components::micro(
            tr(
                state,
                "Imagery: NASA / SDO. Interface built with Rune.",
                "Imágenes: NASA / SDO. Interfaz construida con Rune.",
            ),
            t.text_4,
        ));

    let body = col()
        .key("about-body")
        .w_full()
        .gap(14.0)
        .child(hero)
        .child(row().w_full().gap(14.0).child(mission).child(instruments))
        .child(credits);

    col()
        .key("about-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(header)
        .child(body.h_full().min_h(0.0).scroll(ScrollSpec {
            stick_bottom: false,
        }))
}

fn point(state: &State, index: &str, text: &str) -> rune::El {
    let t = crate::ui::theme::Tokens::get();
    let _ = state;
    row()
        .w_full()
        .gap(12.0)
        .child(
            el().w(26.0)
                .h(20.0)
                .child(components::label(index, theme::MONO_SM, t.teal)),
        )
        .child(
            components::label(text.to_string(), theme::BODY_SM, t.text_2)
                .wrap()
                .grow(1.0),
        )
}
