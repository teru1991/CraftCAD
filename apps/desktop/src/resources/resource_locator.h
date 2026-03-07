#pragma once

#include <QDir>
#include <QString>
#include <QStringList>

struct ResourceRoot {
    bool ok = false;
    QString reasonCode;
    QString message;
    QDir root;
    QStringList searched;
    QStringList missingRequired;
};

ResourceRoot locateResourceRoot();
QStringList resolveRequired(const QDir& root);
QString describeResourceRoot(const ResourceRoot& root);
