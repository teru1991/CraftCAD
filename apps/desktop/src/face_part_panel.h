#pragma once
#include "doc_store.h"
#include <QJsonArray>
#include <QWidget>

class CanvasWidget;
class QListWidget;

class FacePartPanel : public QWidget {
public:
    FacePartPanel(DocStore* store, CanvasWidget* canvas, QWidget* parent = nullptr);

private:
    void detectFaces();
    void onFaceSelectionChanged();
    void createPartFromFace();
    void refreshParts();
    void editSelectedPart();
    void deleteSelectedPart();
    void exportBom();

    DocStore* store_;
    CanvasWidget* canvas_;
    QListWidget* facesList_;
    QListWidget* partsList_;
    QJsonArray faces_;
};
