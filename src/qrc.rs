qrc!(qml_resources,
    "/" {
        "qml/uttesla.qml",
        "qml/MainPage.qml",
    },
);

pub fn load() {
    qml_resources();
}
