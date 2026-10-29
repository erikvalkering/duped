#include <jni.h>
#include <stdio.h>
#include <android/log.h>

JNIEXPORT void JNICALL Java_com_example_duped_MainActivity_onActivityResult(
    JNIEnv* env, jobject thiz, jint requestCode, jint resultCode, jobject data) {
    if (requestCode == 42 && resultCode == -1 && data != NULL) {
        jclass intentClass = (*env)->GetObjectClass(env, data);
        jmethodID getData = (*env)->GetMethodID(env, intentClass, "getData", "()Landroid/net/Uri;");
        jobject uri = (*env)->CallObjectMethod(env, data, getData);
        if (uri) {
            jclass uriClass = (*env)->GetObjectClass(env, uri);
            jmethodID toString = (*env)->GetMethodID(env, uriClass, "toString", "()Ljava/lang/String;");
            jstring uriStr = (jstring)(*env)->CallObjectMethod(env, uri, toString);
            const char* c_uri = (*env)->GetStringUTFChars(env, uriStr, 0);
            __android_log_print(ANDROID_LOG_INFO, "duped", "Picked folder URI: %s", c_uri);
            (*env)->ReleaseStringUTFChars(env, uriStr, c_uri);
        }
    }
}
