#include "view3d_widget.h"

#include <QMouseEvent>
#include <QPainter>
#include <QWheelEvent>
#include <QtMath>
#include <algorithm>
#include <limits>

View3dWidget::View3dWidget(QWidget* parent) : QOpenGLWidget(parent) {}

void View3dWidget::setPartBoxes(const QVector<PartBox>& boxes) {
    boxes_ = boxes;
    if (!boxes_.isEmpty() && selectedPartId_.isEmpty()) {
        selectedPartId_ = boxes_.front().partId;
        emit selectedPartChanged(selectedPartId_);
    }
    update();
}

void View3dWidget::initializeGL() {
    initializeOpenGLFunctions();
    glEnable(GL_DEPTH_TEST);
}

void View3dWidget::resizeGL(int w, int h) {
    glViewport(0, 0, w, h);
}

QMatrix4x4 View3dWidget::projection() const {
    QMatrix4x4 p;
    const float aspect = height() > 0 ? static_cast<float>(width()) / static_cast<float>(height()) : 1.0f;
    p.perspective(45.0f, aspect, 0.1f, 10000.0f);
    return p;
}

QMatrix4x4 View3dWidget::view() const {
    QMatrix4x4 v;
    const float yaw = qDegreesToRadians(yawDeg_);
    const float pitch = qDegreesToRadians(pitchDeg_);
    QVector3D eye(
        target_.x() + distance_ * std::cos(pitch) * std::cos(yaw),
        target_.y() + distance_ * std::cos(pitch) * std::sin(yaw),
        target_.z() + distance_ * std::sin(pitch));
    v.lookAt(eye, target_, QVector3D(0, 0, 1));
    return v;
}

QVector3D View3dWidget::cameraPosition() const {
    const float yaw = qDegreesToRadians(yawDeg_);
    const float pitch = qDegreesToRadians(pitchDeg_);
    return QVector3D(
        target_.x() + distance_ * std::cos(pitch) * std::cos(yaw),
        target_.y() + distance_ * std::cos(pitch) * std::sin(yaw),
        target_.z() + distance_ * std::sin(pitch));
}

QVector3D View3dWidget::unprojectToWorld(float x, float y, float zNdc) const {
    QMatrix4x4 inv = (projection() * view()).inverted();
    QVector4D p(x, y, zNdc, 1.0f);
    QVector4D w = inv * p;
    if (std::abs(w.w()) < 1e-8f) return QVector3D();
    return QVector3D(w.x() / w.w(), w.y() / w.w(), w.z() / w.w());
}

bool View3dWidget::rayIntersectsAabb(const QVector3D& o, const QVector3D& d, const PartBox& box, float* tHit) const {
    const QVector3D bmin = box.min;
    const QVector3D bmax = box.max;
    float tmin = -std::numeric_limits<float>::infinity();
    float tmax = std::numeric_limits<float>::infinity();

    for (int i = 0; i < 3; ++i) {
        const float oi = o[i];
        const float di = d[i];
        const float mn = bmin[i];
        const float mx = bmax[i];
        if (std::abs(di) < 1e-8f) {
            if (oi < mn || oi > mx) return false;
            continue;
        }
        float t1 = (mn - oi) / di;
        float t2 = (mx - oi) / di;
        if (t1 > t2) std::swap(t1, t2);
        tmin = std::max(tmin, t1);
        tmax = std::min(tmax, t2);
        if (tmin > tmax) return false;
    }

    if (tmax < 0.0f) return false;
    *tHit = tmin >= 0.0f ? tmin : tmax;
    return true;
}

void View3dWidget::updateSelectionAt(const QPoint& pos) {
    if (boxes_.isEmpty() || width() <= 0 || height() <= 0) return;

    const float x = (2.0f * pos.x()) / float(width()) - 1.0f;
    const float y = 1.0f - (2.0f * pos.y()) / float(height());

    const QVector3D nearP = unprojectToWorld(x, y, -1.0f);
    const QVector3D farP = unprojectToWorld(x, y, 1.0f);
    QVector3D dir = farP - nearP;
    if (dir.lengthSquared() < 1e-12f) return;
    dir.normalize();

    float bestT = std::numeric_limits<float>::infinity();
    QString best;
    for (const auto& box : boxes_) {
        float t = 0.0f;
        if (rayIntersectsAabb(nearP, dir, box, &t) && t < bestT) {
            bestT = t;
            best = box.partId;
        }
    }

    if (!best.isEmpty() && best != selectedPartId_) {
        selectedPartId_ = best;
        emit selectedPartChanged(selectedPartId_);
        update();
    }
}

QPointF View3dWidget::projectPoint(const QVector3D& p, const QMatrix4x4& vp) const {
    QVector4D clip = vp * QVector4D(p, 1.0f);
    if (std::abs(clip.w()) < 1e-8f) return QPointF(-1e6, -1e6);
    QVector3D ndc(clip.x() / clip.w(), clip.y() / clip.w(), clip.z() / clip.w());
    const float sx = (ndc.x() * 0.5f + 0.5f) * width();
    const float sy = (1.0f - (ndc.y() * 0.5f + 0.5f)) * height();
    return QPointF(sx, sy);
}

void View3dWidget::drawBoxEdges(QPainter& painter, const PartBox& box, const QMatrix4x4& vp, bool selected) {
    QVector3D mn = box.min;
    QVector3D mx = box.max;
    QVector<QVector3D> c{
        {mn.x(), mn.y(), mn.z()}, {mx.x(), mn.y(), mn.z()}, {mx.x(), mx.y(), mn.z()}, {mn.x(), mx.y(), mn.z()},
        {mn.x(), mn.y(), mx.z()}, {mx.x(), mn.y(), mx.z()}, {mx.x(), mx.y(), mx.z()}, {mn.x(), mx.y(), mx.z()},
    };
    static const int e[12][2] = {{0,1},{1,2},{2,3},{3,0},{4,5},{5,6},{6,7},{7,4},{0,4},{1,5},{2,6},{3,7}};

    QColor color = QColor::fromRgba(box.color);
    if (selected) color = QColor(255, 255, 0);
    QPen pen(color);
    pen.setWidth(selected ? 3 : 1);
    painter.setPen(pen);

    for (const auto& edge : e) {
        QPointF a = projectPoint(c[edge[0]], vp);
        QPointF b = projectPoint(c[edge[1]], vp);
        painter.drawLine(a, b);
    }
}

void View3dWidget::paintGL() {
    glClearColor(0.08f, 0.08f, 0.1f, 1.0f);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    QPainter painter(this);
    painter.setRenderHint(QPainter::Antialiasing, true);

    const QMatrix4x4 vp = projection() * view();
    for (const auto& box : boxes_) {
        drawBoxEdges(painter, box, vp, box.partId == selectedPartId_);
    }

    painter.end();
}

void View3dWidget::mousePressEvent(QMouseEvent* event) {
    lastPos_ = event->pos();
    if (event->button() == Qt::LeftButton) {
        updateSelectionAt(event->pos());
    }
}

void View3dWidget::mouseMoveEvent(QMouseEvent* event) {
    const QPoint delta = event->pos() - lastPos_;
    lastPos_ = event->pos();

    if (event->buttons() & Qt::LeftButton) {
        yawDeg_ += delta.x() * 0.4f;
        pitchDeg_ = std::clamp(pitchDeg_ - delta.y() * 0.4f, -89.0f, 89.0f);
        update();
    } else if (event->buttons() & (Qt::RightButton | Qt::MiddleButton)) {
        const float panScale = distance_ * 0.0015f;
        target_.setX(target_.x() - delta.x() * panScale);
        target_.setY(target_.y() + delta.y() * panScale);
        update();
    }
}

void View3dWidget::wheelEvent(QWheelEvent* event) {
    const float steps = event->angleDelta().y() / 120.0f;
    distance_ = std::clamp(distance_ * (1.0f - steps * 0.1f), 10.0f, 5000.0f);
    update();
}
