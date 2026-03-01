#include "arc_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
#include <cmath>
#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
ArcTool::ArcTool(DocStore* store, Camera* camera):store_(store),camera_(camera){}
void ArcTool::onPointerDown(const QPointF& p){ WVec2 w=camera_->screenToWorld(p); snap_=computeSnap(*store_,w,std::nullopt); WVec2 pt=snap_.best?snap_.best->point:w; if(!c_) c_=pt; else if(!s_) s_=pt; else e_=pt; }
void ArcTool::onPointerMove(const QPointF& p){ if(!c_||!s_) return; WVec2 w=camera_->screenToWorld(p); snap_=computeSnap(*store_,w,c_); e_=snap_.best?snap_.best->point:w; }
void ArcTool::onPointerUp(const QPointF&){ if(!c_||!s_||!e_) return; double sx=s_->x-c_->x, sy=s_->y-c_->y; double ex=e_->x-c_->x, ey=e_->y-c_->y; double r=std::sqrt(sx*sx+sy*sy); if(r<=0){ QMessageBox::warning(nullptr,"Arc","GEOM_DEGENERATE"); c_.reset(); s_.reset(); e_.reset(); return; }
    double a0=std::atan2(sy,sx), a1=std::atan2(ey,ex); if(angleLock_){ double step=M_PI/12.0; a1=std::round(a1/step)*step; }
    if(auto deg=numeric_.value()) a1=a0 + (*deg)*M_PI/180.0;
    QByteArray doc=store_->documentJson().toUtf8(); auto d=QJsonDocument::fromJson(doc).object(); QString layer=d.value("layers").toArray().first().toObject().value("id").toString();
    QJsonObject params{{"mode","Center"},{"c",QJsonObject{{"x",c_->x},{"y",c_->y}}},{"r",r},{"start_angle",a0},{"end_angle",a1},{"ccw",true}};
    QByteArray lb=layer.toUtf8(), pb=QJsonDocument(params).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8();
    QString env=take(craftcad_history_apply_create_arc(store_->historyHandle(),doc.constData(),lb.constData(),pb.constData(),eb.constData()));
    auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Arc",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
    else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact)));
    c_.reset(); s_.reset(); e_.reset(); numeric_.clear();
}
void ArcTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_A) angleLock_=!angleLock_; if(e->key()==Qt::Key_Escape){ c_.reset(); s_.reset(); e_.reset(); numeric_.clear(); } numeric_.handleKey(e->key(),e->text()); }
void ArcTool::renderOverlay(QPainter& p){ if(!c_||!s_||!e_) return; p.setPen(QPen(QColor(120,255,120),1,Qt::DashLine)); QPointF c=camera_->worldToScreen(*c_); QPointF s=camera_->worldToScreen(*s_); QPointF e=camera_->worldToScreen(*e_); p.drawLine(c,s); p.drawLine(c,e); }
