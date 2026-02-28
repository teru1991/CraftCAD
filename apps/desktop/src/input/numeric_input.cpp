#include "numeric_input.h"
#include <QKeyEvent>

void NumericInput::handleKey(int key, const QString& text) {
    if ((key >= Qt::Key_0 && key <= Qt::Key_9) || key == Qt::Key_Period) buffer_.append(text);
    if (key == Qt::Key_Backspace && !buffer_.isEmpty()) buffer_.chop(1);
}
void NumericInput::clear() { buffer_.clear(); }
std::optional<double> NumericInput::value() const {
    bool ok = false;
    double v = buffer_.toDouble(&ok);
    if (!ok) return std::nullopt;
    return v;
}
