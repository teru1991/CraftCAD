#pragma once

#include "ffi/craftcad_ffi.h"
#include <QMatrix4x4>
#include <QOpenGLFunctions>
#include <QOpenGLWidget>
#include <QPoint>
#include <QString>
#include <QVector>
#include <QVector3D>

class QPainter;

class View3dWidget : public QOpenGLWidget, protected QOpenGLFunctions {
    Q_OBJECT
public:
    struct PartBox {
        QString partId;
        QVector3D min;
        QVector3D max;
        QRgb color;
    };

    explicit View3dWidget(QWidget* parent = nullptr);
    void setPartBoxes(const QVector<PartBox>& boxes);

signals:
    void selectedPartChanged(const QString& partId);

protected:
    void initializeGL() override;
    void resizeGL(int w, int h) override;
    void paintGL() override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;

private:
    QVector<PartBox> boxes_;
    QString selectedPartId_;

    float yawDeg_{30.0f};
    float pitchDeg_{25.0f};
    float distance_{400.0f};
    QVector3D target_{50.0f, 50.0f, 0.0f};
    QPoint lastPos_;

    QMatrix4x4 projection() const;
    QMatrix4x4 view() const;
    QVector3D cameraPosition() const;
    QVector3D unprojectToWorld(float x, float y, float zNdc) const;
    bool rayIntersectsAabb(const QVector3D& rayOrigin, const QVector3D& rayDir, const PartBox& box, float* tHit) const;
    void updateSelectionAt(const QPoint& pos);
    QPointF projectPoint(const QVector3D& p, const QMatrix4x4& vp) const;
    void drawBoxEdges(QPainter& painter, const PartBox& box, const QMatrix4x4& vp, bool selected);
};
