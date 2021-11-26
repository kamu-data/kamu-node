import {Injectable} from '@angular/core';
import {Apollo} from 'apollo-angular';
import {map, tap} from 'rxjs/operators';
import {ApolloQueryResult, DocumentNode, gql} from '@apollo/client/core';
import {Observable, of, Subject, throwError} from 'rxjs';
import {AuthQueryResult, UserInterface} from '../interface/auth.interface';
import {HttpClient} from '@angular/common/http';
import {Router} from '@angular/router';
import {userResponse} from "./mock.user";
import {subscribe} from "graphql";
import AppValues from "../common/app.values";
@Injectable()
export class AuthApi {
    private user: UserInterface | {};
    private isAuthenticated: boolean;
    private userChanges$: Subject<UserInterface | {}> = new Subject<UserInterface | {}>();
    constructor(private apollo: Apollo, private httpClient: HttpClient, private router: Router) {}


    public get onUserChanges(): Observable<UserInterface | {}> {
       return this.userChanges$.asObservable();
    }
    public userChange(user: UserInterface | {}) {
        this.user = user;
        this.userChanges$.next(user);
    }

    public get userModal() {
        return this.user;
    }

    public set isAuthUser(isAuthenticated: boolean) {
        this.isAuthenticated = isAuthenticated;
    }
    public get isAuthUser(): boolean {
        return this.isAuthenticated;
    }

    public getUserInfoAndToken(code: string): Observable<void> {
        return this.getAccessToken(code).pipe(map((accessToken: string) => {
            localStorage.setItem(AppValues.localStorageCode, code);
            localStorage.setItem(AppValues.localStorageAccessToken, accessToken);


            this.isAuthUser = true;
            // this.authApi.getUser(accessToken);
        }, (err: any) => {
            this.isAuthUser = false;
            localStorage.removeItem(AppValues.localStorageAccessToken);
            AuthApi.handleError(err);
        }));
    }

    public getAccessToken(code: string): Observable<string> {

        const GET_DATA: DocumentNode = gql`mutation GithubLogin {
  auth {
    githubLogin(code: "${code}") {
      token {
        accessToken
        scope
        tokenType
      }
      accountInfo {
        login
        email
        name
        avatarUrl
        gravatarId
      }
    }
  }
}`;

        /* eslint-disable  @typescript-eslint/no-explicit-any */
        // @ts-ignore
        return this.apollo.mutate({mutation: GET_DATA}).pipe(map((result: ApolloQueryResult<any>) => {
                const login = result as AuthQueryResult;
                if (login.data) {
                    const accountInfo: UserInterface = login.data.auth.githubLogin.accountInfo;
                    this.userChange(accountInfo);
                    return login.data.auth.githubLogin.token.accessToken;
                }
            }));

        // this.userChange(userResponse);
        // return of('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');
    }


    public getUser(token: string = ''): void {
        const localStorageAccessToken: string | null = localStorage.getItem(AppValues.localStorageAccessToken);
        const accessToken: string = (token === '' && localStorageAccessToken) ? localStorageAccessToken : token;

      //   this.getUserRequest(accessToken).subscribe((user: UserInterface) => {
      //       debugger
      //       this.userChange(user);
      //       localStorage.setItem('access_token', accessToken);
      //       this.router.navigate(['/']);
      // });
  }

  public logOut(): void {
        this.userChange({});
        localStorage.removeItem(AppValues.localStorageAccessToken);
        localStorage.removeItem(AppValues.localStorageCode);
        this.router.navigate(['/']);
  }

  static handleError(error: Response): Observable<never> {
      return throwError(`GitHub ${error.statusText || 'Server error'}`);
  }
}
