#include "canvas_widget.h"
#include "doc_store.h"
#include "face_part_panel.h"
#include "ffi/craftcad_ffi.h"
#include <QApplication>
#include <QFile>
#include <QFileDialog>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMainWindow>
#include <QDockWidget>
#include <QMenuBar>
#include <QMessageBox>

static bool runExportAction(
    QWidget* parent,
    DocStore& store,
    char* (*ffi_fn)(const char*, const char*),
    const QJsonObject& options,
    const QString& filter,
    const QString& defaultName
) {
    auto docJson = QJsonDocument(store.document).toJson(QJsonDocument::Compact);
    auto opts = QJsonDocument(options).toJson(QJsonDocument::Compact);
    char* out = ffi_fn(docJson.constData(), opts.constData());
    if (!out) {
        QMessageBox::warning(parent, "Export failed", "EXPORT_PDF_FAILED");
        return false;
    }
    QByteArray env(out);
    craftcad_free_string(out);
    auto root = QJsonDocument::fromJson(env).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(parent, "Export failed", QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
        return false;
    }
    auto data = root.value("data").toObject();
    auto bytes = QByteArray::fromBase64(data.value("bytes_base64").toString().toUtf8());
    auto path = QFileDialog::getSaveFileName(parent, "Save Export", defaultName, filter);
    if (path.isEmpty()) return false;
    QFile f(path);
    if (!f.open(QIODevice::WriteOnly)) {
        QMessageBox::warning(parent, "Export failed", "EXPORT_IO_WRITE_FAILED");
        return false;
    }
    f.write(bytes);
    QMessageBox::information(parent, "Export", "Export completed.");
    return true;
}

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    QString path;
    if (app.arguments().size() > 1) path = app.arguments()[1];
    if (path.isEmpty()) path = QFileDialog::getOpenFileName(nullptr, "Open .diycad", QString(), "DIYCAD Files (*.diycad)");
    if (path.isEmpty()) return 0;

    DocStore store;
    QString err;
    if (!store.loadDiycad(path, &err)) {
        QMessageBox::critical(nullptr, "Failed to open project", err);
        return 1;
    }

    QMainWindow w;
    w.setWindowTitle("CraftCAD Desktop");
    auto* canvas = new CanvasWidget(&store);
    w.setCentralWidget(canvas);
    auto* dock = new QDockWidget("Faces/Parts", &w);
    dock->setWidget(new FacePartPanel(&store, canvas));
    w.addDockWidget(Qt::RightDockWidgetArea, dock);

    auto* exportMenu = w.menuBar()->addMenu("&Export");
    auto* tiledAction = exportMenu->addAction("Tiled PDF (1:1)");
    auto* drawingAction = exportMenu->addAction("Drawing PDF");
    auto* svgAction = exportMenu->addAction("SVG");

    QObject::connect(tiledAction, &QAction::triggered, [&]() {
        QJsonObject opts{{"page_size", "A4"}, {"orientation", "Portrait"}, {"margin_mm", 10.0},
                         {"include_crop_marks", true}, {"include_scale_gauge", true},
                         {"title", "CraftCAD Tiled"}, {"include_metadata", true}};
        runExportAction(&w, store, craftcad_export_tiled_pdf, opts, "PDF Files (*.pdf)", "tiled.pdf");
    });
    QObject::connect(drawingAction, &QAction::triggered, [&]() {
        QJsonObject opts{{"title", "CraftCAD Drawing"}};
        runExportAction(&w, store, craftcad_export_drawing_pdf, opts, "PDF Files (*.pdf)", "drawing.pdf");
    });
    QObject::connect(svgAction, &QAction::triggered, [&]() {
        QJsonObject opts{{"precision", 3}, {"include_parts", true}, {"include_entities", true}};
        runExportAction(&w, store, craftcad_export_svg, opts, "SVG Files (*.svg)", "drawing.svg");
    });

    w.resize(1200, 800);
    w.show();
    return app.exec();
}
