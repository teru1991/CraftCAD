#include "canvas_widget.h"
#include "doc_store.h"
#include "face_part_panel.h"
#include "ffi/craftcad_ffi.h"
#include "view3d_widget.h"
#include <QApplication>
#include <QCheckBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QDockWidget>
#include <QFile>
#include <QFileDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QMainWindow>
#include <QMenuBar>
#include <QMessageBox>
#include <QStandardPaths>
#include <QStatusBar>
#include <QTimer>
#include <QVBoxLayout>
#include <algorithm>
#include <cstdio>

static QString take(char* ptr) {
    if (!ptr) return {};
    QString s = QString::fromUtf8(ptr);
    craftcad_free_string(ptr);
    return s;
}

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

static bool runExportAction(QWidget* parent,
                            DocStore& store,
                            char* (*ffi_fn)(const char*, const char*),
                            const QJsonObject& options,
                            const QString& filter,
                            const QString& defaultName) {
    auto docJson = store.documentJson().toUtf8();
    auto opts = QJsonDocument(options).toJson(QJsonDocument::Compact);
    QString env = take(ffi_fn(docJson.constData(), opts.constData()));
    auto root = QJsonDocument::fromJson(env.toUtf8()).object();
    if (!root.value("ok").toBool()) {
        QMessageBox::warning(parent, "UI.ERROR.EXPORT.FAILED", localizeReason(root.value("reason").toObject()));
        return false;
    }
    auto data = root.value("data").toObject();
    auto bytes = QByteArray::fromBase64(data.value("bytes_base64").toString().toUtf8());
    auto path = QFileDialog::getSaveFileName(parent, "UI.DIALOG.EXPORT.SAVE", defaultName, filter);
    if (path.isEmpty()) return false;
    QFile f(path);
    if (!f.open(QIODevice::WriteOnly)) {
        QMessageBox::warning(parent, "UI.ERROR.EXPORT.FAILED", "EXPORT_IO_WRITE_FAILED");
        return false;
    }
    f.write(bytes);
    QMessageBox::information(parent, "Export", "UI.EXPORT.COMPLETED");
    return true;
}

static QVector<View3dWidget::PartBox> loadView3dPartBoxes(const QString& path, QString* error) {
    QVector<View3dWidget::PartBox> out;
    craftcad_part_box_t* ptr = nullptr;
    size_t len = 0;
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_view3d_get_part_boxes(p.constData(), &ptr, &len);
    if (rc != 0) {
        if (error) *error = take(craftcad_last_error_message());
        return out;
    }

    out.reserve(static_cast<qsizetype>(len));
    for (size_t i = 0; i < len; ++i) {
        const craftcad_part_box_t& b = ptr[i];
        const char* id = reinterpret_cast<const char*>(b.part_id_utf8);
        View3dWidget::PartBox box;
        box.partId = QString::fromUtf8(id);
        box.min = QVector3D(b.aabb.min_x, b.aabb.min_y, b.aabb.min_z);
        box.max = QVector3D(b.aabb.max_x, b.aabb.max_y, b.aabb.max_z);
        box.color = static_cast<QRgb>(b.color_rgba);
        out.push_back(box);
    }
    craftcad_view3d_free_part_boxes(ptr, len);
    std::sort(out.begin(), out.end(), [](const auto& a, const auto& b) { return a.partId < b.partId; });
    return out;
}

static int runSmokeView3d(const QString& path) {
    QString err;
    auto boxes = loadView3dPartBoxes(path, &err);
    if (!err.isEmpty()) {
        std::fprintf(stderr, "VIEW3D_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }
    const QString first = boxes.isEmpty() ? QString("none") : boxes.first().partId;
    std::fprintf(stdout, "VIEW3D_SMOKE_OK parts=%d first_part=%s\n", boxes.size(), first.toUtf8().constData());
    return 0;
}

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    const QStringList args = app.arguments();
    const int smokeIdx = args.indexOf("--smoke-view3d");
    if (smokeIdx >= 0) {
        if (smokeIdx + 1 >= args.size()) {
            std::fprintf(stderr, "VIEW3D_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeView3d(args.at(smokeIdx + 1));
    }

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

    auto* view3dDock = new QDockWidget("3D", &w);
    auto* view3d = new View3dWidget(&w);
    view3dDock->setWidget(view3d);
    w.addDockWidget(Qt::LeftDockWidgetArea, view3dDock);

    auto* selectedLabel = new QLabel("Selected PartId: (none)", &w);
    w.statusBar()->addPermanentWidget(selectedLabel);
    QObject::connect(view3d, &View3dWidget::selectedPartChanged, [&selectedLabel](const QString& id) {
        selectedLabel->setText(QString("Selected PartId: %1").arg(id));
    });

    const auto boxes = loadView3dPartBoxes(path, &err);
    if (!err.isEmpty()) {
        QMessageBox::warning(&w, "3D View", QString("Failed to load 3D part boxes: %1").arg(err));
    }
    view3d->setPartBoxes(boxes);

    auto* exportMenu = w.menuBar()->addMenu("&Export");
    auto* tiledAction = exportMenu->addAction("Tiled PDF (1:1)");
    auto* drawingAction = exportMenu->addAction("Drawing PDF");
    auto* svgAction = exportMenu->addAction("SVG");
    auto* helpMenu = w.menuBar()->addMenu("&Help");
    auto* diagAction = helpMenu->addAction("Export Diagnostic Pack");

    QObject::connect(tiledAction, &QAction::triggered, [&]() {
        QJsonObject opts{{"page_size", "A4"},
                         {"orientation", "Portrait"},
                         {"margin_mm", 10.0},
                         {"include_crop_marks", true},
                         {"include_scale_gauge", true},
                         {"title", "CraftCAD Tiled"},
                         {"include_metadata", true}};
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
        dlg.setWindowTitle("UI.DIALOG.SUPPORT.TITLE");
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
        QJsonObject opts{{"max_logs", 100},
                         {"latest_n", 100},
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
