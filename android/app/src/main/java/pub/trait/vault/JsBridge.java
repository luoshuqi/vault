package pub.trait.vault;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.webkit.JavascriptInterface;
import android.widget.Toast;

import java.net.Inet4Address;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.net.SocketException;
import java.util.Calendar;
import java.util.Enumeration;

public class JsBridge {
    public static final int CODE_CHOOSE_IMPORT_FILE = 1;

    public static final int CODE_SAVE_EXPORT_FILE = 2;

    private final MainActivity activity;

    public JsBridge(MainActivity activity) {
        this.activity = activity;
    }

    @JavascriptInterface
    public void toast(String msg, boolean length_long) {
        Toast.makeText(activity, msg, length_long ? Toast.LENGTH_LONG : Toast.LENGTH_SHORT).show();
    }

    @JavascriptInterface
    public String getIp() {
        try {
            for (Enumeration<NetworkInterface> interfaces = NetworkInterface.getNetworkInterfaces(); interfaces.hasMoreElements(); ) {
                NetworkInterface iface = interfaces.nextElement();
                if (iface.isUp() || !iface.isLoopback()) {
                    for (Enumeration<InetAddress> addresses = iface.getInetAddresses(); addresses.hasMoreElements(); ) {
                        InetAddress address = addresses.nextElement();
                        if (address instanceof Inet4Address) {
                            return address.getHostAddress();
                        }
                    }
                }

            }
            return null;
        } catch (SocketException e) {
            e.printStackTrace();
            return null;
        }
    }

    @JavascriptInterface
    public void copyToClipboard(String content) {
        ClipboardManager cm = (ClipboardManager) activity.getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = ClipData.newPlainText("", content);
        cm.setPrimaryClip(data);
    }

    @JavascriptInterface
    public String getCacheDir() {
        return activity.getApplicationContext().getCacheDir().getAbsolutePath();
    }

    @JavascriptInterface
    public void saveExportFile(String file) {
        Intent intent = new Intent(Intent.ACTION_CREATE_DOCUMENT);
        intent.setType("text/plain");
        intent.putExtra(Intent.EXTRA_TITLE, makeExportFilename());
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        activity.startActivityForResult(intent, CODE_SAVE_EXPORT_FILE);
        activity.setTmpExportFile(file);
    }

    @JavascriptInterface
    public void chooseImportFile() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.setType("text/plain");
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        activity.startActivityForResult(intent, CODE_CHOOSE_IMPORT_FILE);
    }

    private String makeExportFilename() {
        Calendar calendar = Calendar.getInstance();

        int year = calendar.get(Calendar.YEAR);
        int month = calendar.get(Calendar.MONTH) + 1;
        int day = calendar.get(Calendar.DATE);
        int hour = calendar.get(Calendar.HOUR_OF_DAY);
        int min = calendar.get(Calendar.MINUTE);
        int sec = calendar.get(Calendar.SECOND);

        return String.format("密码_%d%02d%02d_%02d%02d%02d.txt", year, month, day, hour, min, sec);
    }
}
