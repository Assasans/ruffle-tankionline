package flash.globalization
{
    import __ruffle__.stub_constructor;
    import __ruffle__.stub_method;

    public final class DateTimeFormatter {

        public var actualLocaleIDName:String;
        public var lastOperationStatus:String;
        public var requestedLocaleIDName:String;

        public function DateTimeFormatter(requestedLocaleIDName:String, dateStyle:String = "long", timeStyle:String = "long") {
            stub_constructor("flash.globalization.DateTimeFormatter");
        }

        public function format(dateTime:Date):String {
            stub_method("flash.globalization.DateTimeFormatter", "format");
        }

        public function formatUTC(dateTime:Date):String {
            stub_method("flash.globalization.DateTimeFormatter", "formatUTC");
        }

        public static function getAvailableLocaleIDNames():Vector.<String> {
            stub_method("flash.globalization.DateTimeFormatter", "getAvailableLocaleIDNames");
        }

        public function getDateStyle():String {
            stub_method("flash.globalization.DateTimeFormatter", "getDateStyle");
        }

        public function getDateTimePattern():String {
            stub_method("flash.globalization.DateTimeFormatter", "getDateTimePattern");
        }

        public function getFirstWeekday():int {
            stub_method("flash.globalization.DateTimeFormatter", "getFirstWeekday");
        }

        public function getMonthNames(nameStyle:String = "full", context:String = "standalone"):Vector.<String> {
            stub_method("flash.globalization.DateTimeFormatter", "getMonthNames");
        }

        public function getTimeStyle():String {
            stub_method("flash.globalization.DateTimeFormatter", "getTimeStyle");
        }

        public function getWeekdayNames(nameStyle:String = "full", context:String = "standalone"):Vector.<String> {
            stub_method("flash.globalization.DateTimeFormatter", "getWeekdayNames");
        }

        public function setDateTimePattern(pattern:String):void {
            stub_method("flash.globalization.DateTimeFormatter", "setDateTimePattern");
        }

        public function setDateTimeStyles(dateStyle:String, timeStyle:String):void {
            stub_method("flash.globalization.DateTimeFormatter", "setDateTimeStyles");
        }
    }
}
