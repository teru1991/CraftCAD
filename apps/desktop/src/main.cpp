#include "canvas_widget.h"
#include "doc_store.h"
#include "face_part_panel.h"
#include "ffi/craftcad_ffi.h"
#include <QApplication>
#include <QFile>
#include <QFileDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMainWindow>
#include <QDockWidget>
#include <QMenuBar>
#include <QMessageBox>
#include <QStandardPaths>
#include <QTimer>
#include <QDialog>
#include <QVBoxLayout>
#include <QDialogButtonBox>
#include <QCheckBox>

static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }

static QString localizeReason(const QJsonObject& reasonObj) {
    const QString key = reasonObj.value("user_msg_key").toString();
    if (key.isEmpty()) return QString::fromUtf8(QJsonDocument(reasonObj).toJson(QJsonDocument::Compact));
    const QByteArray kb = key.toUtf8();
    const QByteArray pb = QJsonDocument(reasonObj.value("params").toObject()).toJson(QJsonDocument::Compact);
    const QByteArray lb = QByteArray("ja-JP");
    QString env = take(craftcad_i18n_resolve_message(kb.constData(), pb.constData(), lb.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) return QString::fromUtf8(QJsonDocument(reasonObj).toJson(QJsonDocument::Compact));
    return root.value("data").toObject().value("message").toString();
}

static bool runExportAction(
    QWidget* parent,
    DocStore& store,
    char* (*ffi_fn)(const char*, const char*),
    const QJsonObject& options,
    const QString& filter,
    const QString& defaultName
) {
    auto docJson = store.documentJson().toUtf8();
    auto opts = QJsonDocument(options).toJson(QJsonDocument::Compact);
    QString env = take(ffi_fn(docJson.constData(), opts.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(parent, "Export failed", localizeReason(root.value("reason").toObject()));
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

    if (store.hasRecoverySnapshot()) {
        auto answer = QMessageBox::question(nullptr, "Recovery found", "A crash-recovery autosave exists. Restore it?");
        if (answer == QMessageBox::Yes) {
            if (!store.loadRecoverySnapshot(&err)) {
                QMessageBox::warning(nullptr, "Recovery failed", err);
            }
        }
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
    auto* helpMenu = w.menuBar()->addMenu("&Help");
    auto* diagAction = helpMenu->addAction("Export Diagnostic Pack");

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
    QObject::connect(diagAction, &QAction::triggered, [&]() {
        QDialog dlg(&w);
        dlg.setWindowTitle("Diagnostic Pack Options");
        auto* layout = new QVBoxLayout(&dlg);
        auto* includeDoc = new QCheckBox("Include document snapshot (contains project data)");
        includeDoc->setChecked(false);
        auto* includeSystem = new QCheckBox("Include system information");
        includeSystem->setChecked(false);
        layout->addWidget(includeDoc);
        layout->addWidget(includeSystem);
        auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
        layout->addWidget(buttons);
        QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
        QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);
        if (dlg.exec() != QDialog::Accepted) return;

        QJsonArray logs;
        for (const auto& line : store.latestReasonLogs(100)) logs.append(line);
        QJsonObject opts{{"max_logs", 100}, {"latest_n", 100},
                         {"include_doc", includeDoc->isChecked()},
                         {"include_doc_snapshot", includeDoc->isChecked()},
                         {"include_system", includeSystem->isChecked()},
                         {"reason_logs", logs},
                         {"locale", "ja-JP"},
                         {"eps", QJsonDocument::fromJson(store.epsPolicyJson().toUtf8()).object()}};
        runExportAction(&w, store, craftcad_export_diagnostic_pack, opts, "ZIP Files (*.zip)", "diagnostic_pack.zip");
    });

    QTimer autosave;
    autosave.setInterval(5000);
    QObject::connect(&autosave, &QTimer::timeout, [&]() {
        QString r;
        store.autosaveNow(&r);
    });
    autosave.start();

    QObject::connect(&app, &QCoreApplication::aboutToQuit, [&]() {
        QString r;
        store.autosaveNow(&r);
    });

    w.resize(1200, 800);
    w.show();
    return app.exec();
}
