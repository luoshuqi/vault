package pub.trait.vault;

import java.util.function.Function;

public class Vault {

    static {
        System.loadLibrary("vault");
    }

    /**
     * 启动 server
     *
     * @param dataDir  数据目录
     * @param listener 启动回调, 参数为 server url
     */
    public static native void start(String dataDir, Function<String, Void> listener);

    /**
     * 停止 server
     */
    public static native void stop();
}
