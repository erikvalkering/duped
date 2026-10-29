package com.example.duped;

import android.app.NativeActivity;
import android.content.Intent;
import android.util.Log;

public class MainActivity extends NativeActivity {
    private static final String TAG = "duped";

    static {
        // Matches target/release/libduped_android.so.
        System.loadLibrary("duped_android");
    }

    private static native void nativeOnActivityResult(
        int requestCode,
        int resultCode,
        Intent data
    );

    @Override
    protected void onActivityResult(
        int requestCode,
        int resultCode,
        Intent data
    ) {
        super.onActivityResult(requestCode, resultCode, data);

        Log.d(
            TAG,
            "onActivityResult request=" + requestCode
                + " result=" + resultCode
        );

        nativeOnActivityResult(requestCode, resultCode, data);
    }
}
