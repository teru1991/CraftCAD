#include "canvas_widget.h"
#include "doc_store.h"
#include "face_part_panel.h"
#include "ffi/craftcad_ffi.h"
#include "view3d_widget.h"
#include "resources/resource_locator.h"
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
#include <QLineEdit>
#include <QListWidget>
#include <QMainWindow>
#include <QMenuBar>
#include <QPushButton>
#include <QMessageBox>
#include <QStandardPaths>
#include <QStatusBar>
#include <QSpinBox>
#include <QStringList>
#include <QShortcut>
#include <QFormLayout>
#include <QTimer>
#include <QVBoxLayout>
#include <algorithm>
#include <cstdio>
#include <functional>

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






static int runSmokeResources() {
    const ResourceRoot located = locateResourceRoot();

    QJsonObject evidence;
    evidence.insert("root", located.root.absolutePath());

    QJsonArray searched;
    for (const QString& s : located.searched) searched.append(s);
    evidence.insert("searched", searched);

    QJsonArray missing;
    for (const QString& m : located.missingRequired) missing.append(m);
    evidence.insert("missing", missing);

    QJsonObject out;
    out.insert("kind", "smoke");
    out.insert("name", "resources");
    out.insert("ok", located.ok);
    out.insert("reason_code", located.ok ? QJsonValue() : QJsonValue(located.reasonCode));
    out.insert("message", located.ok ? QString("resources preflight ok") : located.message);
    out.insert("evidence", evidence);

    const QByteArray line = QJsonDocument(out).toJson(QJsonDocument::Compact);
    std::fprintf(stdout, "%s\n", line.constData());
    return located.ok ? 0 : 1;
}

static int runSmokeEstimateLite(const QString& path) {
    craftcad_estimate_lite_hash_t est{};
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_estimate_lite_hash(p.constData(), &est);
    if (rc != 0) {
        const QString err = take(craftcad_last_error_message());
        std::fprintf(stderr, "ESTIMATE_LITE_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }

    const char* hash = reinterpret_cast<const char*>(est.hash_hex);
    const char* first = reinterpret_cast<const char*>(est.first_material_id_utf8);
    std::fprintf(
        stdout,
        "ESTIMATE_LITE_SMOKE_OK hash=%s items=%zu first_material=%s\n",
        hash,
        est.item_count,
        est.item_count > 0 ? first : "none");
    return 0;
}

static int runSmokeProjectionLite(const QString& path) {
    craftcad_projection_lite_hashes_t hashes{};
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_projection_lite_hashes(p.constData(), &hashes);
    if (rc != 0) {
        const QString err = take(craftcad_last_error_message());
        std::fprintf(stderr, "PROJ_LITE_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }

    const char* front = reinterpret_cast<const char*>(hashes.front_hash_hex);
    const char* top = reinterpret_cast<const char*>(hashes.top_hash_hex);
    const char* side = reinterpret_cast<const char*>(hashes.side_hash_hex);
    std::fprintf(
        stdout,
        "PROJ_LITE_SMOKE_OK front=%s top=%s side=%s parts=%zu\n",
        front,
        top,
        side,
        hashes.part_count);
    return 0;
}


static int runSmokeFastenerBomLite(const QString& path) {
    craftcad_mfg_hints_lite_hash_t hints{};
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_mfg_hints_lite_hash(p.constData(), &hints);
    if (rc != 0) {
        const QString err = take(craftcad_last_error_message());
        std::fprintf(stderr, "FASTENER_BOM_LITE_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }

    const char* hash = reinterpret_cast<const char*>(hints.hash_hex);
    std::fprintf(stdout, "FASTENER_BOM_LITE_SMOKE_OK hash=%s items=%zu\n", hash, hints.item_count);
    return 0;
}

static int runSmokeMfgHintsLite(const QString& path) {
    craftcad_mfg_hints_lite_hash_t hints{};
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_mfg_hints_lite_hash(p.constData(), &hints);
    if (rc != 0) {
        const QString err = take(craftcad_last_error_message());
        std::fprintf(stderr, "MFG_HINTS_LITE_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }

    const char* hash = reinterpret_cast<const char*>(hints.hash_hex);
    std::fprintf(stdout, "MFG_HINTS_LITE_SMOKE_OK hash=%s items=%zu\n", hash, hints.item_count);
    return 0;
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

static int runSmokeRulesEdge(const QString& path) {
    char* jsonPtr = nullptr;
    size_t len = 0;
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_rules_edge_report(p.constData(), &jsonPtr, &len);
    if (rc != 0) {
        const QString err = take(craftcad_last_error_message());
        std::fprintf(stderr, "RULES_EDGE_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 2;
    }

    const QByteArray payload = QByteArray::fromRawData(jsonPtr, static_cast<qsizetype>(len));
    const QJsonObject root = QJsonDocument::fromJson(payload).object();
    craftcad_rules_edge_free_json(jsonPtr);
    const QJsonArray findings = root.value("findings").toArray();
    int fatals = 0;
    int warns = 0;
    for (const auto& f : findings) {
        const QString sev = f.toObject().value("severity").toString();
        if (sev == "fatal") {
            ++fatals;
        } else if (sev == "warn") {
            ++warns;
        }
    }
    std::fprintf(stdout, "RULES_EDGE_SMOKE_OK fatals=%d warns=%d HAS_FATAL=%d\n", fatals, warns, fatals > 0 ? 1 : 0);
    return 0;
}

static int runSmokeExportPreflight(const QString& path) {
    const QByteArray p = path.toUtf8();
    const int rc = craftcad_export_preflight_check(p.constData());
    if (rc == 0) {
        std::fprintf(stdout, "EXPORT_PREFLIGHT_OK BLOCKED=0\n");
        return 0;
    }
    const QString err = take(craftcad_last_error_message());
    if (rc == 10) {
        std::fprintf(stderr, "EXPORT_PREFLIGHT_BLOCKED BLOCKED=1 report=%s\n", err.toUtf8().constData());
        return 3;
    }
    std::fprintf(stderr, "EXPORT_PREFLIGHT_FAIL error=%s\n", err.toUtf8().constData());
    return 2;
}

static bool loadPartInspectorJson(const QString& projectPath, const QString& partId, QJsonObject* out, QString* err) {
    char* ptr = nullptr;
    size_t len = 0;
    const QByteArray pathUtf8 = projectPath.toUtf8();
    const QByteArray idUtf8 = partId.toUtf8();
    const int rc = craftcad_ssot_get_part(pathUtf8.constData(), idUtf8.constData(), &ptr, &len);
    if (rc != 0) {
        if (err) *err = take(craftcad_last_error_message());
        return false;
    }
    const QByteArray payload = QByteArray::fromRawData(ptr, static_cast<qsizetype>(len));
    const QJsonDocument doc = QJsonDocument::fromJson(payload);
    craftcad_free_string(ptr);
    if (!doc.isObject()) {
        if (err) *err = QStringLiteral("invalid_part_payload");
        return false;
    }
    if (out) *out = doc.object();
    if (err) err->clear();
    return true;
}

static int runSmokeInspectorEdit(const QString& path) {
    QString err;
    const auto boxes = loadView3dPartBoxes(path, &err);
    if (!err.isEmpty() || boxes.isEmpty()) {
        std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=%s\n", (!err.isEmpty() ? err : QStringLiteral("no_parts")).toUtf8().constData());
        return 2;
    }
    const QString partId = boxes.first().partId;
    const QByteArray p = path.toUtf8();
    const QByteArray id = partId.toUtf8();
    const QByteArray newName = QByteArray("smoke_name");

    if (craftcad_ssot_set_part_name(p.constData(), id.constData(), newName.constData()) != 0) {
        std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=%s\n", take(craftcad_last_error_message()).toUtf8().constData());
        return 3;
    }
    if (craftcad_ssot_set_part_quantity(p.constData(), id.constData(), 2) != 0) {
        std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=%s\n", take(craftcad_last_error_message()).toUtf8().constData());
        return 4;
    }

    QJsonObject partObj;
    if (!loadPartInspectorJson(path, partId, &partObj, &err)) {
        std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=%s\n", err.toUtf8().constData());
        return 5;
    }
    if (partObj.value("name").toString() != QStringLiteral("smoke_name") || partObj.value("quantity").toInt() != 2) {
        std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=persistence_mismatch\n");
        return 6;
    }

    std::fprintf(stdout, "INSPECTOR_SMOKE_OK part=%s\n", partId.toUtf8().constData());
    return 0;
}

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);

    const QStringList args = app.arguments();
    const int smokeResourcesIdx = args.indexOf("--smoke-resources");
    if (smokeResourcesIdx >= 0) {
        return runSmokeResources();
    }

    const int smokeEstimateIdx = args.indexOf("--smoke-estimate-lite");
    if (smokeEstimateIdx >= 0) {
        if (smokeEstimateIdx + 1 >= args.size()) {
            std::fprintf(stderr, "ESTIMATE_LITE_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeEstimateLite(args.at(smokeEstimateIdx + 1));
    }
    const int smokeProjectionIdx = args.indexOf("--smoke-projection-lite");
    if (smokeProjectionIdx >= 0) {
        if (smokeProjectionIdx + 1 >= args.size()) {
            std::fprintf(stderr, "PROJ_LITE_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeProjectionLite(args.at(smokeProjectionIdx + 1));
    }

    const int smokeFastenerIdx = args.indexOf("--smoke-fastener-bom-lite");
    if (smokeFastenerIdx >= 0) {
        if (smokeFastenerIdx + 1 >= args.size()) {
            std::fprintf(stderr, "FASTENER_BOM_LITE_SMOKE_FAIL error=missing_project_path
");
            return 2;
        }
        return runSmokeFastenerBomLite(args.at(smokeFastenerIdx + 1));
    }

    const int smokeHintsIdx = args.indexOf("--smoke-mfg-hints-lite");
    if (smokeHintsIdx >= 0) {
        if (smokeHintsIdx + 1 >= args.size()) {
            std::fprintf(stderr, "MFG_HINTS_LITE_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeMfgHintsLite(args.at(smokeHintsIdx + 1));
    }

    const int smokeRulesIdx = args.indexOf("--smoke-rules-edge");
    if (smokeRulesIdx >= 0) {
        if (smokeRulesIdx + 1 >= args.size()) {
            std::fprintf(stderr, "RULES_EDGE_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeRulesEdge(args.at(smokeRulesIdx + 1));
    }

    const int smokePreflightIdx = args.indexOf("--smoke-export-preflight");
    if (smokePreflightIdx >= 0) {
        if (smokePreflightIdx + 1 >= args.size()) {
            std::fprintf(stderr, "EXPORT_PREFLIGHT_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeExportPreflight(args.at(smokePreflightIdx + 1));
    }

    const int smokeInspectorEditIdx = args.indexOf("--smoke-inspector-edit");
    if (smokeInspectorEditIdx >= 0) {
        if (smokeInspectorEditIdx + 1 >= args.size()) {
            std::fprintf(stderr, "INSPECTOR_SMOKE_FAIL error=missing_project_path\n");
            return 2;
        }
        return runSmokeInspectorEdit(args.at(smokeInspectorEditIdx + 1));
    }

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

    const auto boxes = loadView3dPartBoxes(path, &err);
    if (!err.isEmpty()) {
        QMessageBox::warning(&w, "3D View", QString("Failed to load 3D part boxes: %1").arg(err));
    }
    view3d->setPartBoxes(boxes);

    auto* inspectorDock = new QDockWidget("Inspector", &w);
    auto* inspectorWidget = new QWidget(inspectorDock);
    auto* inspectorForm = new QFormLayout(inspectorWidget);
    auto* inspectorPartId = new QLabel("(none)", inspectorWidget);
    auto* inspectorMaterial = new QLabel("(none)", inspectorWidget);
    auto* inspectorThickness = new QLabel("(none)", inspectorWidget);
    auto* inspectorName = new QLineEdit(inspectorWidget);
    auto* inspectorQty = new QSpinBox(inspectorWidget);
    inspectorQty->setRange(1, 1'000'000);
    inspectorForm->addRow("PartId", inspectorPartId);
    inspectorForm->addRow("Material", inspectorMaterial);
    inspectorForm->addRow("Thickness", inspectorThickness);
    inspectorForm->addRow("Name", inspectorName);
    inspectorForm->addRow("Quantity", inspectorQty);
    inspectorWidget->setLayout(inspectorForm);
    inspectorDock->setWidget(inspectorWidget);
    w.addDockWidget(Qt::RightDockWidgetArea, inspectorDock);

    QString selectedPartId;
    bool inspectorUpdating = false;

    auto refresh3d = [&]() {
        QString refreshErr;
        const auto refreshed = loadView3dPartBoxes(path, &refreshErr);
        if (!refreshErr.isEmpty()) {
            w.statusBar()->showMessage(QString("3D refresh failed: %1").arg(refreshErr), 4000);
            return;
        }
        view3d->setPartBoxes(refreshed);
    };

    auto loadPartIntoInspector = [&](const QString& partId) {
        inspectorUpdating = true;
        selectedPartId = partId;
        inspectorPartId->setText(partId.isEmpty() ? QStringLiteral("(none)") : partId);
        inspectorName->setEnabled(!partId.isEmpty());
        inspectorQty->setEnabled(!partId.isEmpty());

        if (partId.isEmpty()) {
            inspectorName->clear();
            inspectorQty->setValue(1);
            inspectorMaterial->setText("(none)");
            inspectorThickness->setText("(none)");
            inspectorUpdating = false;
            return;
        }

        QJsonObject partObj;
        QString loadErr;
        if (!loadPartInspectorJson(path, partId, &partObj, &loadErr)) {
            w.statusBar()->showMessage(QString("Inspector load failed: %1").arg(loadErr), 5000);
            inspectorUpdating = false;
            return;
        }

        inspectorName->setText(partObj.value("name").toString());
        inspectorQty->setValue(partObj.value("quantity").toInt(1));
        const QJsonObject material = partObj.value("material").toObject();
        inspectorMaterial->setText(material.value("name").toString("(none)"));
        if (material.value("thickness_mm").isDouble()) {
            inspectorThickness->setText(QString::number(material.value("thickness_mm").toDouble()) + " mm");
        } else {
            inspectorThickness->setText("(none)");
        }
        inspectorUpdating = false;
    };

    QObject::connect(view3d, &View3dWidget::selectedPartChanged, [&selectedLabel, &loadPartIntoInspector](const QString& id) {
        selectedLabel->setText(QString("Selected PartId: %1").arg(id));
        loadPartIntoInspector(id);
    });

    QObject::connect(inspectorName, &QLineEdit::editingFinished, [&]() {
        if (inspectorUpdating || selectedPartId.isEmpty()) return;
        const QByteArray p = path.toUtf8();
        const QByteArray id = selectedPartId.toUtf8();
        const QByteArray name = inspectorName->text().toUtf8();
        const int rc = craftcad_ssot_set_part_name(p.constData(), id.constData(), name.constData());
        if (rc != 0) {
            QMessageBox::warning(&w, "Inspector", take(craftcad_last_error_message()));
            loadPartIntoInspector(selectedPartId);
            return;
        }
        w.statusBar()->showMessage("Saved", 2000);
        refresh3d();
        loadPartIntoInspector(selectedPartId);
    });

    QObject::connect(inspectorQty, qOverload<int>(&QSpinBox::valueChanged), [&](int value) {
        if (inspectorUpdating || selectedPartId.isEmpty()) return;
        const QByteArray p = path.toUtf8();
        const QByteArray id = selectedPartId.toUtf8();
        const int rc = craftcad_ssot_set_part_quantity(p.constData(), id.constData(), static_cast<uint32_t>(value));
        if (rc != 0) {
            QMessageBox::warning(&w, "Inspector", take(craftcad_last_error_message()));
            loadPartIntoInspector(selectedPartId);
            return;
        }
        w.statusBar()->showMessage("Saved", 2000);
        refresh3d();
    });

    auto* fileMenu = w.menuBar()->addMenu("&File");
    auto* openAction = fileMenu->addAction("Open Project…");
    auto* saveAction = fileMenu->addAction("Save Project");
    auto* exportMenu = w.menuBar()->addMenu("&Export");
    auto* tiledAction = exportMenu->addAction("Tiled PDF (1:1)");
    auto* drawingAction = exportMenu->addAction("Drawing PDF");
    auto* svgAction = exportMenu->addAction("SVG");
    auto* helpMenu = w.menuBar()->addMenu("&Help");
    auto* diagAction = helpMenu->addAction("Export Diagnostic Pack");

    auto reloadProject = [&](const QString& newPath) {
        QString loadErr;
        if (!store.loadDiycad(newPath, &loadErr)) {
            QMessageBox::critical(&w, "Open failed", loadErr);
            return false;
        }
        path = newPath;
        refresh3d();
        loadPartIntoInspector(QString());
        w.statusBar()->showMessage("Project loaded", 2000);
        return true;
    };

    QObject::connect(openAction, &QAction::triggered, [&]() {
        const QString newPath = QFileDialog::getOpenFileName(&w, "Open .diycad", QString(), "DIYCAD Files (*.diycad)");
        if (!newPath.isEmpty()) {
            reloadProject(newPath);
        }
    });
    QObject::connect(saveAction, &QAction::triggered, [&]() {
        QString saveErr;
        if (store.autosaveNow(&saveErr)) {
            w.statusBar()->showMessage("Project saved", 2000);
        } else {
            QMessageBox::warning(&w, "Save failed", saveErr);
        }
    });

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

    struct PaletteCommand {
        QString name;
        std::function<void()> run;
    };

    QVector<PaletteCommand> paletteCommands{
        {"Open Project…", [&]() { openAction->trigger(); }},
        {"Save Project", [&]() { saveAction->trigger(); }},
        {"Run: Smoke View3D", [&]() { w.statusBar()->showMessage(runSmokeView3d(path) == 0 ? "OK" : "FAIL", 2500); }},
        {"Run: Smoke Projection Lite", [&]() { w.statusBar()->showMessage(runSmokeProjectionLite(path) == 0 ? "OK" : "FAIL", 2500); }},
        {"Run: Smoke Estimate Lite", [&]() { w.statusBar()->showMessage(runSmokeEstimateLite(path) == 0 ? "OK" : "FAIL", 2500); }},
        {"Run: Smoke Fastener BOM Lite", [&]() { w.statusBar()->showMessage("Not implemented", 2500); }},
        {"Run: Smoke Rules Edge", [&]() { w.statusBar()->showMessage(runSmokeRulesEdge(path) == 0 ? "OK" : "FAIL", 2500); }},
        {"Run: Smoke Mfg Hints Lite", [&]() { w.statusBar()->showMessage(runSmokeMfgHintsLite(path) == 0 ? "OK" : "FAIL", 2500); }},
        {"Run: Smoke ViewPack Inspect", [&]() { w.statusBar()->showMessage("Use craftcad-viewpack-inspect CLI", 3000); }},
    };

    auto showPalette = [&]() {
        QDialog dlg(&w);
        dlg.setWindowTitle("Command Palette");
        auto* layout = new QVBoxLayout(&dlg);
        auto* input = new QLineEdit(&dlg);
        auto* list = new QListWidget(&dlg);
        layout->addWidget(input);
        layout->addWidget(list);

        auto refreshList = [&]() {
            list->clear();
            const QString q = input->text().trimmed();
            for (const auto& cmd : paletteCommands) {
                if (q.isEmpty() || cmd.name.contains(q, Qt::CaseInsensitive)) {
                    list->addItem(cmd.name);
                }
            }
            if (list->count() > 0) list->setCurrentRow(0);
        };

        QObject::connect(input, &QLineEdit::textChanged, &dlg, refreshList);
        QObject::connect(list, &QListWidget::itemDoubleClicked, &dlg, [&](QListWidgetItem*) { dlg.accept(); });
        refreshList();

        if (dlg.exec() != QDialog::Accepted) return;
        const auto* item = list->currentItem();
        if (!item) return;
        const QString selected = item->text();
        for (const auto& cmd : paletteCommands) {
            if (cmd.name == selected) {
                cmd.run();
                break;
            }
        }
    };

    auto* paletteShortcut = new QShortcut(QKeySequence(QStringLiteral("Ctrl+K")), &w);
    QObject::connect(paletteShortcut, &QShortcut::activated, &w, showPalette);
    auto* paletteSlashShortcut = new QShortcut(QKeySequence(QStringLiteral("/")), &w);
    QObject::connect(paletteSlashShortcut, &QShortcut::activated, &w, showPalette);

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
