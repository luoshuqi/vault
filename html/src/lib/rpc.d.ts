declare class Rpc {
    // 是否设置了主密码
    is_master_password_set(): Promise<boolean>;

    // 设置主密码
    set_master_password(master_password: string): Promise<void>;

    // 验证主密码
    verify_master_password(master_password: string): Promise<boolean>;

    // 获取所有密码
    list_password(master_password: string): Promise<Array<Item>>;

    // 获取密码
    get_password(master_password: string, id: number): Promise<Password>;

    /**
     * 导入密码
     * @param master_password
     * @param decrypt_password 解密导入数据的密码
     * @param source 文件或者要导入的数据
     */
    import_password(master_password: string, decrypt_password: string | null, source: string | Array<Array<String>>): Promise<Count>;

    /**
     * 导出密码
     * @param master_password
     * @param file 文件， 如果不为 null，导出到此文件，否则返回导出的数据
     */
    export_password(master_password: string, file: string | null): Promise<Array<Array<String>> | null>;

    // 删除密码
    delete_password(master_password: string, id: number): Promise<void>;

    // 生成密码
    make_password(option: PasswordOption): Promise<String>;

    // 添加密码
    add_password(master_password: String, name: String, password: String): Promise<void>;

    // 更新密码
    update_password(master_password: String, id: number, name: String, password: String): Promise<void>;

    // 修改主密码
    change_password(master_password: String, new_password: String): Promise<void>;

    // 获取可从网络访问的端口号
    get_network_port(): Promise<number|null>;

    // 开启网络访问
    enable_network_access(): Promise<number>;

    // 关闭网络访问
    disable_network_access(): Promise<void>;
}

export declare var rpc: Rpc;

declare class Item {
    public id: number;
    public name: string;
}

declare class Count {
    ignore: number;
    insert: number;
}

declare class Password {
    name: string;
    password: string;
}

declare class PasswordOption {
    len: number;
    uppercase: boolean;
    lowercase: boolean;
    digit: boolean;
    special: boolean;
}
