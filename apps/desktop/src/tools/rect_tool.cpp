#include "rect_tool.h"
#include "../ffi/craftcad_ffi.h"
#include <QJsonDocument>
#include <QMessageBox>
#include <cmath>

static QString take(char* ptr){ if(!ptr) return {}; QString s=QString::fromUtf8(ptr); craftcad_free_string(ptr); return s; }
RectTool::RectTool(DocStore* store, Camera* camera):store_(store),camera_(camera){}
void RectTool::onPointerDown(const QPointF& s){ WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,std::nullopt); p0_=snap_.best?snap_.best->point:w; p1_=p0_; }
void RectTool::onPointerMove(const QPointF& s){ if(!p0_) return; WVec2 w=camera_->screenToWorld(s); snap_=computeSnap(*store_,w,p0_); p1_=snap_.best?snap_.best->point:w; if(lockH_) p1_->y=p0_->y; if(lockV_) p1_->x=p0_->x; }
void RectTool::onPointerUp(const QPointF&){ if(!p0_||!p1_) return; if(lockH_&&lockV_){ QMessageBox::warning(nullptr,"Rect","DRAW_CONSTRAINT_CONFLICT"); p0_.reset(); p1_.reset(); return; }
    auto len=numeric_.value(); if(len){ double dx=p1_->x-p0_->x,dy=p1_->y-p0_->y; double n=std::sqrt(dx*dx+dy*dy); if(n>0){ dx/=n; dy/=n; p1_={p0_->x+dx*(*len),p0_->y+dy*(*len)}; }}
    QByteArray doc=store_->documentJson().toUtf8(); auto d=QJsonDocument::fromJson(doc).object(); QString layer=d.value("layers").toArray().first().toObject().value("id").toString();
    QJsonObject params{{"mode","TwoPoint"},{"p0",QJsonObject{{"x",p0_->x},{"y",p0_->y}}},{"p1",QJsonObject{{"x",p1_->x},{"y",p1_->y}}},{"corner","Sharp"}};
    QByteArray lb=layer.toUtf8(), pb=QJsonDocument(params).toJson(QJsonDocument::Compact), eb=store_->epsPolicyJson().toUtf8();
    QString env=take(craftcad_history_apply_create_rect(store_->historyHandle(),doc.constData(),lb.constData(),pb.constData(),eb.constData()));
    auto root=QJsonDocument::fromJson(env.toUtf8()).object(); if(!root.value("ok").toBool()) QMessageBox::warning(nullptr,"Rect",QString::fromUtf8(QJsonDocument(root.value("reason").toObject()).toJson()));
    else store_->setDocumentJson(QString::fromUtf8(QJsonDocument(root.value("data").toObject().value("document").toObject()).toJson(QJsonDocument::Compact)));
    p0_.reset(); p1_.reset(); numeric_.clear();
}
void RectTool::onKeyPress(QKeyEvent* e){ if(e->key()==Qt::Key_H) lockH_=!lockH_; if(e->key()==Qt::Key_V) lockV_=!lockV_; if(e->key()==Qt::Key_Escape){ p0_.reset(); p1_.reset(); numeric_.clear(); } numeric_.handleKey(e->key(),e->text()); }
void RectTool::renderOverlay(QPainter& p){ if(!p0_||!p1_) return; p.setPen(QPen(QColor(120,255,120),1,Qt::DashLine)); QPolygonF poly; poly<<camera_->worldToScreen(*p0_)<<camera_->worldToScreen({p1_->x,p0_->y})<<camera_->worldToScreen(*p1_)<<camera_->worldToScreen({p0_->x,p1_->y}); p.drawPolygon(poly); }
