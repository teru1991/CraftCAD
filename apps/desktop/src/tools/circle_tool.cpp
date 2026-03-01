#include "circle_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
#include <cmath>
static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
CircleTool::CircleTool(DocStore* store, Camera* camera):store_(store),camera_(camera){}
void CircleTool::onPointerDown(const QPointF& s){ WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,std::nullopt); if(!c_) c_=snap_.best?snap_.best->point:w; else edge_=snap_.best?snap_.best->point:w; }
void CircleTool::onPointerMove(const QPointF& s){ if(!c_) return; WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,c_); edge_=snap_.best?snap_.best->point:w; }
void CircleTool::onPointerUp(const QPointF&){ if(!c_||!edge_) return; double dx=edge_->x-c_->x,dy=edge_->y-c_->y,r=std::sqrt(dx*dx+dy*dy); if(auto n=numeric_.value()) r=*n; if(r<=0){ QMessageBox::warning(nullptr,"Circle","DRAW_INVALID_NUMERIC"); c_.reset(); edge_.reset(); return; }
    QByteArray doc=store_->documentJson().toUtf8(); auto d=QJsonDocument::fromJson(doc).object(); QString layer=d.value("layers").toArray().first().toObject().value("id").toString();
    QJsonObject params{{"mode","CenterRadius"},{"c",QJsonObject{{"x",c_->x},{"y",c_->y}}},{"r",r}};
    QByteArray lb=layer.toUtf8(), pb=QJsonDocument(params).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8();
    QString env=take(craftcad_history_apply_create_circle(store_->historyHandle(),doc.constData(),lb.constData(),pb.constData(),eb.constData()));
    auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Circle",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
    else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact)));
    c_.reset(); edge_.reset(); numeric_.clear();
}
void CircleTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_Escape){ c_.reset(); edge_.reset(); numeric_.clear(); } numeric_.handleKey(e->key(),e->text()); }
void CircleTool::renderOverlay(QPainter& p){ if(!c_||!edge_) return; double dx=edge_->x-c_->x,dy=edge_->y-c_->y,r=std::sqrt(dx*dx+dy*dy); if(auto n=numeric_.value()) r=*n; QPointF cc=camera_->worldToScreen(*c_); double rr=r*camera_->zoom; p.setPen(QPen(QColor(120,255,120),1,Qt::DashLine)); p.drawEllipse(cc,rr,rr); }
