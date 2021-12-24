package pub.trait.vault;

import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.webkit.WebView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class MainActivity extends AppCompatActivity {
    private WebView webView;

    private String tmpExportFile;

    public MainActivity() {
        super();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        webView = findViewById(R.id.webview);
        webView.getSettings().setJavaScriptEnabled(true);
        webView.addJavascriptInterface(new JsBridge(this), "bridge");

        String dataDir = getApplicationContext().getFilesDir().getAbsolutePath() + "/vault";
        new Thread(() -> {
            try {
                Vault.start(dataDir, url -> {
                    runOnUiThread(() -> webView.loadUrl(url));
                    return null;
                });
            } catch (Throwable e) {
                runOnUiThread(() -> showError(e));
            }
        }).start();
    }

    public void setTmpExportFile(String tmpExportFile) {
        this.tmpExportFile = tmpExportFile;
    }

    private void showError(Throwable e) {
        Toast.makeText(this, "出错了(" + e.getMessage() + ")", Toast.LENGTH_LONG).show();
    }

    @Override
    public void onBackPressed() {
        try {
            webView.evaluateJavascript("window.backPressedListener()", value -> {
                if (value.equals("true")) {
                    finish();
                }
            });
        } catch (Throwable e) {
            e.printStackTrace();
            finish();
        }
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, @Nullable Intent data) {
        switch (requestCode) {
            case JsBridge.CODE_CHOOSE_IMPORT_FILE:
                resolveChooseImportFile(resultCode == RESULT_OK && data != null ? data.getData() : null);
                break;
            case JsBridge.CODE_SAVE_EXPORT_FILE:
                if (resultCode == RESULT_OK && data != null) {
                    saveExportFile(tmpExportFile, data.getData());
                }
                new File(tmpExportFile).delete();
                tmpExportFile = null;
                break;
        }
        super.onActivityResult(requestCode, resultCode, data);
    }

    private void saveExportFile(String source, Uri dest) {
        try (FileInputStream in = new FileInputStream(source);
             OutputStream out = getContentResolver().openOutputStream(dest, "wt")) {
            copy(in, out);
            Toast.makeText(this, "导出成功", Toast.LENGTH_SHORT).show();
        } catch (IOException e) {
            showError(e);
        }
    }

    private void resolveChooseImportFile(@Nullable Uri uri) {
        if (uri == null) {
            webView.evaluateJavascript("window.completeChooseImportFile(null)", null);
            return;
        }

        String path = getCacheDir().getAbsolutePath() + "/" + System.currentTimeMillis();
        try (InputStream in = getContentResolver().openInputStream(uri);
             FileOutputStream out = new FileOutputStream(path)) {
            copy(in, out);
            webView.evaluateJavascript("window.completeChooseImportFile(\"" + path + "\")", null);
        } catch (IOException e) {
            showError(e);
            webView.evaluateJavascript("window.completeChooseImportFile(null)", null);
        }
    }

    private void copy(InputStream in, OutputStream out) throws IOException {
        int length;
        byte[] bytes = new byte[8192];
        while ((length = in.read(bytes)) != -1) {
            out.write(bytes, 0, length);
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        Vault.stop();
    }
}